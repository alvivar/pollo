use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::usize;

enum Command {
    Add(String, TcpStream),
    Call(String),
}

pub struct Subs {
    registry: HashMap<String, Vec<TcpStream>>,
    tx: Sender<Command>,
    rx: Receiver<Command>,
}

impl Subs {
    pub fn new() -> Subs {
        let registry = HashMap::<String, Vec<Sub>>::new();
        let (tx, rx) = channel::<Command>();

        Subs { registry, tx, rx }
    }

    pub fn handle(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Command::Add(key, id, socket)) => {
                    let conns = self.registry.entry(key).or_insert_with(Vec::new);
                    conns.push(Sub::new(id, socket));
                }

                Ok(Command::Call(key)) => {
                    self.registry.entry(key).or_insert_with(Vec::new);
                }

                Err(_) => {}
            }
        }
    }
}
