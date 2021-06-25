use std::net::{SocketAddr, TcpStream};
use std::usize;

pub struct Connection {
    pub id: usize,
    pub socket: TcpStream,
    pub addr: SocketAddr,
    pub handshake: bool,
}

impl Connection {
    pub fn new(id: usize, socket: TcpStream, addr: SocketAddr) -> Connection {
        Connection {
            id,
            socket,
            addr,
            handshake: false,
        }
    }
}
