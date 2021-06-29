use std::{io::Write, sync::mpsc::Sender};

use crate::{conn::Connection, ready};

pub struct Writer {
    ready_tx: Sender<ready::Cmd>,
}

impl Writer {
    pub fn new(ready_tx: Sender<ready::Cmd>) -> Writer {
        Writer { ready_tx }
    }

    pub fn handle(self, mut conn: Connection, msg: String) {
        if let Err(err) = conn.socket.write(msg.as_bytes()) {
            // @todo Tell subscriptions to clean up this id.
            println!("Socket #{} error, dropped: {}", conn.id, err);
        } else {
            self.ready_tx.send(ready::Cmd::Write(conn)).unwrap();
        }
    }
}
