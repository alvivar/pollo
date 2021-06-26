use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Cmd {
    Add(String, usize, TcpStream),
    Del(String, usize),
    Call(String, String),
}

struct Sub {
    id: usize,
    socket: TcpStream,
}

pub struct Subs {
    registry: HashMap<String, Vec<Sub>>,
    pub tx: Sender<Cmd>,
    rx: Receiver<Cmd>,
}

impl Subs {
    pub fn new() -> Subs {
        let registry = HashMap::<String, Vec<Sub>>::new();
        let (tx, rx) = channel::<Cmd>();

        Subs { registry, tx, rx }
    }

    pub fn handle(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Cmd::Add(key, id, socket)) => {
                    let subs = self.registry.entry(key.to_owned()).or_insert_with(Vec::new);

                    if subs.iter().any(|x| x.id == id) {
                        continue;
                    }

                    subs.push(Sub { id, socket })
                }

                Ok(Cmd::Del(key, id)) => {
                    let subs = self.registry.entry(key.to_owned()).or_insert_with(Vec::new);
                    subs.retain(|x| x.id != id);
                }

                Ok(Cmd::Call(key, value)) => {
                    let subs = self.registry.entry(key.to_owned()).or_insert_with(Vec::new);

                    let mut broken = Vec::<usize>::new();
                    for (i, sub) in subs.iter_mut().enumerate() {
                        let msg = format!("{} {}", key, value);
                        if let Err(err) = sub.socket.write(msg.as_bytes()) {
                            println!("Sub broken, dropping socket #{} from #{}: {}", i, key, err);
                            broken.push(i);
                        }
                    }

                    for &index in broken.iter().rev() {
                        subs.swap_remove(index);
                    }
                }

                Err(err) => panic!("The sub channel failed: {}", err),
            }
        }
    }
}
