use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::Work;

pub enum Cmd {
    Add(String, usize),
    Del(String, usize),
    Call(String, String),
}

pub struct Subs {
    registry: HashMap<String, Vec<usize>>,
    work_tx: Sender<Work>,
    pub tx: Sender<Cmd>,
    rx: Receiver<Cmd>,
}

impl Subs {
    pub fn new(work_tx: Sender<Work>) -> Subs {
        let registry = HashMap::<String, Vec<usize>>::new();
        let (tx, rx) = channel::<Cmd>();

        Subs {
            registry,
            work_tx,
            tx,
            rx,
        }
    }

    pub fn handle(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Cmd::Add(key, id)) => {
                    let subs = self.registry.entry(key.to_owned()).or_insert_with(Vec::new);

                    if subs.iter().any(|x| x == &id) {
                        continue;
                    }

                    subs.push(id)
                }

                Ok(Cmd::Del(key, id)) => {
                    let subs = self.registry.entry(key.to_owned()).or_insert_with(Vec::new);
                    subs.retain(|x| x != &id);
                }

                Ok(Cmd::Call(key, value)) => {
                    if let Some(subs) = self.registry.get(&key) {
                        for &id in subs {
                            let msg = format!("{} {}", key, value);
                            self.work_tx.send(Work::Write(id, msg)).unwrap();
                        }
                    }
                }

                Err(err) => panic!("The sub channel failed: {}", err),
            }
        }
    }
}
