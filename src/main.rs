use std::{
    collections::HashMap,
    io,
    net::TcpListener,
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

use polling::{Event, Poller};

mod conn;
mod parse;
mod pool;
mod reader;
mod ready;
mod subs;

use conn::Connection;
use pool::ThreadPool;
use reader::Reader;
use ready::Ready;
use subs::Subs;

fn main() -> io::Result<()> {
    // The server and the smol Poller.
    let server = TcpListener::bind("0.0.0.0:1984")?;
    server.set_nonblocking(true)?;

    let poller = Poller::new()?;
    poller.add(&server, Event::readable(0))?;
    let poller = Arc::new(poller);

    let conns = HashMap::<usize, Connection>::new();
    let conns = Arc::new(Mutex::new(conns));

    // Subscriptions thread
    let mut subs = Subs::new();
    let subs_tx = subs.tx.clone();

    thread::spawn(move || subs.handle());

    // Thread that re-register the connection for more reading events.
    let ready_poller = poller.clone();
    let ready_conns = conns.clone();
    let ready = Ready::new(ready_poller, ready_conns);
    let ready_tx = ready.tx.clone();

    thread::spawn(move || ready.handle());

    // The thread pool that handles reading the connection and calling
    // subscriptions accordinly.
    let mut work = ThreadPool::new(4);
    let (reader_tx, work_rx) = channel::<Connection>();
    let reader_rx = Arc::new(Mutex::new(work_rx));

    for _ in 0..work.size() {
        let reader_rx = reader_rx.clone();
        let reader = Reader::new(reader_rx);

        let subs_tx = subs_tx.clone();
        let ready_tx = ready_tx.clone();

        work.submit(move || reader.handle(subs_tx, ready_tx));
    }

    // Connections and events via smol Poller.
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

                    // Save the new connection.
                    poller.add(&socket, Event::readable(id))?;
                    conns
                        .lock()
                        .unwrap()
                        .insert(id, Connection::new(id, socket, addr));
                    id += 1;

                    // The server continues listening for more clients, always 0.
                    poller.modify(&server, Event::readable(0))?;
                }

                id => {
                    if let Some(conn) = conns.lock().unwrap().remove(&id) {
                        reader_tx.send(conn).unwrap();
                    }
                }
            }
        }
    }
}
