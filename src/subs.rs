use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::usize;

enum Command {
    Add(usize),
    Delete(usize),
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

    pub fn handle(&self) {
        loop {
            let cmd = self.rx.recv();
            match cmd {
                Ok(Command::Add(id)) => todo!(),

                Ok(Command::Delete(id)) => todo!(),

                Err(_) => todo!(),
            }
        }
    }
}
