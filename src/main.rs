use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::net::TcpListener;
use std::str::from_utf8;
use std::usize;

use polling::{Event, Poller};

mod conn;
use conn::Connection;

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

fn main() -> io::Result<()> {
    let server = TcpListener::bind("0.0.0.0:1984")?;
    server.set_nonblocking(true)?;

    let poller = Poller::new()?;
    poller.add(&server, Event::readable(0))?;

    let mut id: usize = 1;
    let mut conns = HashMap::<usize, Connection>::new();

    let mut events = Vec::new();
    loop {
        events.clear();
        poller.wait(&mut events, None)?;

        for ev in &events {
            match ev.key {
                0 => {
                    println!("Accept on l1");

                    // Handle registry.
                    let (socket, addr) = server.accept()?;
                    socket.set_nonblocking(true)?;

                    // Let's save the connection to read from it later.
                    id += 1;
                    poller.add(&socket, Event::readable(id))?;
                    conns.insert(id, Connection::new(id, socket, addr));

                    // Listen for more clients.
                    poller.modify(&server, Event::readable(0))?;
                }

                id => {
                    if let Some(mut conn) = conns.remove(&id) {
                        let data = match read(&mut conn) {
                            Ok(data) => data,
                            Err(err) => {
                                println!("Connection {} lost: {}", conn.id, err);
                                continue;
                            }
                        };

                        if let Ok(utf8) = from_utf8(&data) {
                            println!("{:?}", utf8);
                        }

                        // Prepare it for more reads!
                        poller.modify(&conn.socket, Event::readable(conn.id))?;
                        conns.insert(id, conn);
                    }
                }
            }
        }
    }
}

fn read(conn: &mut Connection) -> io::Result<Vec<u8>> {
    let mut received_data = vec![0; 1024 * 4];
    let mut bytes_read = 0;

    loop {
        match conn.socket.read(&mut received_data[bytes_read..]) {
            Ok(0) => {
                // Reading 0 bytes means the other side has closed the
                // connection or is done writing, then so are we.
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "0 bytes read"));
            }
            Ok(n) => {
                bytes_read += n;
                if bytes_read == received_data.len() {
                    received_data.resize(received_data.len() + 1024, 0);
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

    received_data.resize(bytes_read, 0);

    Ok(received_data)
}
