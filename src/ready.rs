use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use polling::{Event, Poller};

use crate::conn::Connection;

pub struct Ready {
    poller: Arc<Poller>,
    conns: Arc<Mutex<HashMap<usize, Connection>>>,
    pub tx: Sender<Connection>,
    rx: Receiver<Connection>,
}

impl Ready {
    pub fn new(poller: Arc<Poller>, conns: Arc<Mutex<HashMap<usize, Connection>>>) -> Ready {
        let (tx, rx) = channel::<Connection>();

        Ready {
            poller,
            conns,
            tx,
            rx,
        }
    }

    pub fn handle(self) {
        loop {
            if let Ok(conn) = self.rx.recv() {
                self.poller
                    .modify(&conn.socket, Event::readable(conn.id))
                    .unwrap();

                self.conns.lock().unwrap().insert(conn.id, conn);
            }
        }
    }
}
