use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::usize;

enum Command {
    Add(String, usize),
    Call(String),
}

pub struct Subs {
    registry: HashMap<String, Vec<usize>>,
    tx: Sender<Command>,
    rx: Receiver<Command>,
}

impl Subs {
    pub fn new() -> Subs {
        let registry = HashMap::<String, Vec<usize>>::new();
        let (tx, rx) = channel::<Command>();

        Subs { registry, tx, rx }
    }

    pub fn handle(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Command::Add(key, id)) => {
                    let conns = self.registry.entry(key).or_insert_with(Vec::new);
                    conns.push(id);
                }

                Ok(Command::Call(key)) => {
                    self.registry.entry(key).or_insert_with(Vec::new);
                }

                Err(_) => {}
            }
        }
    }
}
