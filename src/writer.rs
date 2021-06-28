use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub struct Writer {
    registry: Arc<Mutex<HashMap<usize, TcpStream>>>,
}

impl Writer {
    pub fn new(registry: Arc<Mutex<HashMap<usize, TcpStream>>>) -> Writer {
        Writer { registry }
    }

    pub fn register(&mut self, id: usize, socket: TcpStream) {
        self.registry.lock().unwrap().insert(id, socket);
    }

    pub fn handle(self, id: usize, msg: String) {
        let mut registry = self.registry.lock().unwrap();
        if let Some(mut socket) = registry.get(&id) {
            if let Err(err) = socket.write(msg.as_bytes()) {
                // @todo Tell subscriptions to clean up this id.
                println!("Socket #{} error, dropped: {}", id, err);
                registry.remove(&id);
            }
        }
    }
}
