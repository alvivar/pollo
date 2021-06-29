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
mod writer;

use conn::Connection;
use pool::ThreadPool;
use reader::Reader;
use ready::Ready;
use subs::Subs;
use writer::Writer;

pub enum Work {
    Read(Connection),
    Write(Connection, String),
}

fn main() -> io::Result<()> {
    // The server and the smol Poller.
    let server = TcpListener::bind("0.0.0.0:1984")?;
    server.set_nonblocking(true)?;

    let poller = Poller::new()?;
    poller.add(&server, Event::readable(0))?;
    let poller = Arc::new(poller);

    let read_map = HashMap::<usize, Connection>::new();
    let read_map = Arc::new(Mutex::new(read_map));

    let write_map = HashMap::<usize, Connection>::new();
    let write_map = Arc::new(Mutex::new(write_map));

    // Thread that re-register the connection for more reading events.
    let ready_poller = poller.clone();
    let ready_read_map = read_map.clone();
    let ready_write_map = write_map.clone();

    let ready = Ready::new(ready_poller, ready_read_map, ready_write_map);
    let ready_tx = ready.tx.clone();
    thread::spawn(move || ready.handle());

    // The thread pool that handles reading the connection and calling
    // subscriptions accordinly.
    let mut work = ThreadPool::new(4);
    let (work_tx, work_rx) = channel::<Work>();
    let work_rx = Arc::new(Mutex::new(work_rx));

    // Subscriptions thread.
    let subs_write_map = write_map.clone();
    let subs_work_tx = work_tx.clone();
    let mut subs = Subs::new(subs_write_map, subs_work_tx);
    let subs_tx = subs.tx.clone();

    thread::spawn(move || subs.handle());

    for _ in 0..work.size() {
        let work_rx = work_rx.clone();
        let subs_tx = subs_tx.clone();
        let ready_tx = ready_tx.clone();

        work.submit(move || loop {
            let subs_tx = subs_tx.clone();
            let reader_ready_tx = ready_tx.clone();
            let reader = Reader::new(subs_tx, reader_ready_tx);

            let writer_ready_tx = ready_tx.clone();
            let writer = Writer::new(writer_ready_tx);

            match work_rx.lock().unwrap().recv().unwrap() {
                Work::Read(conn) => reader.handle(conn),
                Work::Write(conn, msg) => writer.handle(conn, msg),
            }
        });
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
                    let (read_socket, addr) = server.accept()?;
                    read_socket.set_nonblocking(true)?;
                    let work_socket = read_socket.try_clone().unwrap();

                    println!("Connection #{} from {}", id, addr);

                    // Prepare the reading socket.
                    poller.add(&read_socket, Event::readable(id))?;

                    read_map
                        .lock()
                        .unwrap()
                        .insert(id, Connection::new(id, read_socket, addr));

                    // Register the writing socket.
                    write_map
                        .lock()
                        .unwrap()
                        .insert(id, Connection::new(id, work_socket, addr));

                    // Next one.
                    id += 1;

                    // The server continues listening for more clients, always 0.
                    poller.modify(&server, Event::readable(0))?;
                }

                id => {
                    if let Some(conn) = read_map.lock().unwrap().remove(&id) {
                        work_tx.send(Work::Read(conn)).unwrap();
                    }
                }
            }
        }
    }
}
