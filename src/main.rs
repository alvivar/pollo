use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::net::TcpListener;
use std::str::from_utf8;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use polling::{Event, Poller};

mod conn;
mod parse;
mod pool;
mod subs;

use conn::Connection;
use parse::parse;
use pool::ThreadPool;
use subs::Subs;

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

fn main() -> io::Result<()> {
    // The server and the smol Poller.
    let server = TcpListener::bind("0.0.0.0:1984")?;
    server.set_nonblocking(true)?;

    let poller = Poller::new()?;
    poller.add(&server, Event::readable(0))?;
    let poller = Arc::new(poller);

    let conns = HashMap::<usize, Connection>::new();
    let conns = Arc::new(Mutex::new(conns));

    // Subscriptions
    let mut subs = Subs::new();
    let subs_tx = subs.tx.clone();
    thread::spawn(move || subs.handle());

    // Thread that readys the connections.
    let (ready_tx, ready_rx) = channel::<Connection>();
    let ready_conns = conns.clone();
    let ready_poller = poller.clone();

    thread::spawn(move || loop {
        if let Ok(conn) = ready_rx.try_recv() {
            ready_poller
                .modify(&conn.socket, Event::readable(conn.id))
                .unwrap();

            ready_conns.lock().unwrap().insert(conn.id, conn);
        }
    });

    // The infamous thread pool.
    let mut work = ThreadPool::new(4);
    let (work_tx, work_rx) = channel::<Connection>();
    let work_rx = Arc::new(Mutex::new(work_rx));

    for _ in 0..work.size() {
        let subs_tx = subs_tx.clone();
        let work_rx = work_rx.clone();
        let ready_tx = ready_tx.clone();

        work.submit(move || {
            loop {
                let mut conn = work_rx.lock().unwrap().recv().unwrap();

                let data = match read(&mut conn) {
                    Ok(data) => data,
                    Err(err) => {
                        println!("Connection #{} lost: {}", conn.id, err);
                        continue;
                    }
                };

                // Handle the message as string.
                if let Ok(utf8) = from_utf8(&data) {
                    let msg = parse(utf8);
                    let op = msg.op.as_str();
                    let key = msg.key;
                    let val = msg.value;

                    if !key.is_empty() {
                        match op {
                            // A subscription and a first message.
                            "+" => {
                                let socket = conn.socket.try_clone().unwrap();
                                subs_tx
                                    .send(subs::Cmd::Add(key.to_owned(), conn.id, socket))
                                    .unwrap();

                                if !val.is_empty() {
                                    subs_tx.send(subs::Cmd::Call(key, val)).unwrap()
                                }
                            }

                            // A message to subscriptions.
                            ":" => {
                                subs_tx.send(subs::Cmd::Call(key, val)).unwrap();
                            }

                            // A desubscription and a last message.
                            "-" => {
                                if !val.is_empty() {
                                    subs_tx.send(subs::Cmd::Call(key.to_owned(), val)).unwrap();
                                }

                                subs_tx.send(subs::Cmd::Del(key, conn.id)).unwrap();
                            }

                            _ => (),
                        }
                    }

                    println!("{}: {}", conn.addr, utf8.trim_end());
                }

                // println!("Sending to ready!");
                ready_tx.send(conn).unwrap();
            }
        });
    }

    // Connections and events
    let mut id: usize = 1;
    let mut events = Vec::new();

    loop {
        events.clear();
        poller.wait(&mut events, None)?;

        for ev in &events {
            match ev.key {
                0 => {
                    let (socket, addr) = server.accept()?;
                    socket.set_nonblocking(true)?;

                    println!("Connection #{} from {}", id, addr);

                    // Let's save the connection to read from it later.
                    poller.add(&socket, Event::readable(id))?;
                    conns
                        .lock()
                        .unwrap()
                        .insert(id, Connection::new(id, socket, addr));

                    id += 1;

                    // Continue listen for clients, always 0.
                    poller.modify(&server, Event::readable(0))?;
                }

                id => {
                    if let Some(conn) = conns.lock().unwrap().remove(&id) {
                        work_tx.send(conn).unwrap();
                    }
                }
            }
        }
    }
}

/// Returns the bytes read from a connection.
fn read(conn: &mut Connection) -> io::Result<Vec<u8>> {
    let mut received = vec![0; 1024 * 4];
    let mut bytes_read = 0;

    loop {
        match conn.socket.read(&mut received[bytes_read..]) {
            Ok(0) => {
                // Reading 0 bytes means the other side has closed the
                // connection or is done writing, then so are we.
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "0 bytes read"));
            }
            Ok(n) => {
                bytes_read += n;
                if bytes_read == received.len() {
                    received.resize(received.len() + 1024, 0);
                }
            }
            // Would block "errors" are the OS's way of saying that the
            // connection is not actually ready to perform this I/O operation.
            // @todo Wondering if this should be a panic instead.
            Err(ref err) if would_block(err) => break,
            Err(ref err) if interrupted(err) => continue,
            // Other errors we'll consider fatal.
            Err(err) => return Err(err),
        }
    }

    // let received_data = &received_data[..bytes_read]; // @doubt Using this
    // slice thing and returning with into() versus using the resize? Hm.

    received.resize(bytes_read, 0);

    Ok(received)
}
