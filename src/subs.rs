use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Command {
    Create(String, TcpStream),
    Call(String, String),
}

pub struct Subs {
    registry: HashMap<String, Vec<TcpStream>>,
    pub tx: Sender<Command>,
    pub rx: Receiver<Command>,
}

impl Subs {
    pub fn new() -> Subs {
        let registry = HashMap::<String, Vec<TcpStream>>::new();
        let (tx, rx) = channel::<Command>();

        Subs { registry, tx, rx }
    }

    pub fn handle(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Command::Create(key, socket)) => {
                    let conns = self.registry.entry(key).or_insert_with(Vec::new);
                    conns.push(socket);
                }

                Ok(Command::Call(key, value)) => {
                    let sockets = self.registry.entry(key).or_insert_with(Vec::new);

                    for socket in sockets {
                        socket.write(value.as_bytes()).unwrap();
                        socket.flush().unwrap();
                    }
                }

                Err(_) => {}
            }
        }
    }
}
