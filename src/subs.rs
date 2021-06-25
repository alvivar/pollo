use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Command {
    Add(String, TcpStream),
    Del(),
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
                Ok(Command::Add(key, socket)) => {
                    let conns = self.registry.entry(key).or_insert_with(Vec::new);
                    conns.push(socket);
                }

                Ok(Command::Del()) => {}

                Ok(Command::Call(key, value)) => {
                    let sockets = self.registry.entry(key.to_owned()).or_insert_with(Vec::new);

                    let mut lost_ones = Vec::<usize>::new();
                    for (i, mut socket) in sockets.iter().enumerate() {
                        if let Err(err) = socket.write(value.as_bytes()) {
                            lost_ones.push(i);
                            println!("Sub failed, dropping socket #{} from #{}: {}", i, key, err);
                        }
                    }

                    for &index in lost_ones.iter().rev() {
                        sockets.swap_remove(index);
                    }
                }

                Err(_) => {}
            }
        }
    }
}
