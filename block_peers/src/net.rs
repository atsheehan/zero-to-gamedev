use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::Result;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::grid::Grid;

pub struct Socket {
    socket: UdpSocket,
    buffer: [u8; 1024],
}

impl Socket {
    /// Binds a new UDP socket on any avaliable port.
    pub fn new() -> Result<Self> {
        Self::bind("0.0.0.0:0")
    }

    /// Binds a new UDP socket to the specific socket address.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let buffer = [0; 1024];
        UdpSocket::bind(addr).map(|socket| Socket { socket, buffer })
    }

    pub fn receive<D: DeserializeOwned>(&mut self) -> Result<(SocketAddr, D)> {
        let (bytes_received, source_addr) = match self.socket.recv_from(&mut self.buffer) {
            Ok(result) => result,
            Err(e) => return Err(e),
        };

        let data = &self.buffer[..bytes_received];
        Ok((source_addr, bincode::deserialize(&data).unwrap()))
    }

    pub fn send<A: ToSocketAddrs, S: Serialize>(&mut self, addr: A, message: S) -> Result<()> {
        let serialized_message = bincode::serialize(&message).unwrap();
        self.socket.send_to(&serialized_message, addr)?;

        Ok(())
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
}

impl ClientMessage {
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Ack { grid: Grid },
}

impl ServerMessage {
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
