use std::{
    io::{self, Read},
    str::from_utf8,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{conn::Connection, parse::parse, subs};

pub struct Reader {
    rx: Arc<Mutex<Receiver<Connection>>>,
}

impl Reader {
    pub fn new(rx: Arc<Mutex<Receiver<Connection>>>) -> Reader {
        Reader { rx }
    }

    pub fn handle(&self, subs_tx: Sender<subs::Cmd>, ready_tx: Sender<Connection>) {
        loop {
            let mut conn = self.rx.lock().unwrap().recv().unwrap();

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

            // Re-register the connection for more readings.
            ready_tx.send(conn).unwrap();
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

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
