use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::net::TcpListener;
use std::str::from_utf8;
use std::thread;
use std::usize;

use polling::{Event, Poller};

mod conn;
mod parse;
mod subs;

use conn::Connection;
use parse::parse;
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

    // Subscriptions
    let mut subs = Subs::new();
    let subs_tx = subs.tx.clone();
    thread::spawn(move || subs.handle());

    // Connections and events
    let mut id: usize = 1;
    let mut conns = HashMap::<usize, Connection>::new();
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
                    conns.insert(id, Connection::new(id, socket, addr));
                    id += 1;

                    // Listen for more clients, always using 0.
                    poller.modify(&server, Event::readable(0))?;
                }

                id => {
                    if let Some(mut conn) = conns.remove(&id) {
                        let data = match read(&mut conn) {
                            Ok(data) => data,
                            Err(err) => {
                                println!("Connection #{} lost: {}", conn.id, err);
                                continue;
                            }
                        };

                        // Handle the string message.
                        if let Ok(utf8) = from_utf8(&data) {
                            println!("{}: {}", conn.addr, utf8.trim_end());

                            let msg = parse(utf8);
                            let op = msg.op.as_str();
                            let key = msg.key;
                            let val = msg.value;

                            match op {
                                // A subscription and a first message.
                                "+" => {
                                    let socket = conn.socket.try_clone().unwrap();
                                    subs_tx
                                        .send(subs::Cmd::Add(key.to_owned(), conn.id, socket))
                                        .unwrap();

                                    subs_tx.send(subs::Cmd::Call(key, val)).unwrap()
                                }

                                // A message to subscriptions.
                                ":" => subs_tx.send(subs::Cmd::Call(key, val)).unwrap(),

                                // A desubscription and a last message.
                                "-" => {
                                    subs_tx.send(subs::Cmd::Call(key.to_owned(), val)).unwrap();
                                    subs_tx.send(subs::Cmd::Del(key, conn.id)).unwrap();
                                }

                                _ => (),
                            }
                        }

                        // Prepare for more reads!
                        poller.modify(&conn.socket, Event::readable(conn.id))?;
                        conns.insert(id, conn);
                    }
                }
            }
        }
    }
}

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
            // @todo Do I need this?
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
