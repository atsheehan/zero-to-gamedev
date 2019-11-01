use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::io::{Error, ErrorKind, Result};
use std::mem::transmute;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::constants::PROTOCOL_VERSION;
use crate::grid::{Grid, GridInputEvent};

const BUFFER_SIZE: usize = 4096;

fn protocol_id() -> u32 {
    let mut total: u32 = 0;
    for b in "Block Wars".bytes() {
        total += b as u32;
    }
    total += PROTOCOL_VERSION;
    total
}

pub struct Socket {
    socket: UdpSocket,
    buffer: [u8; BUFFER_SIZE],
}

impl Socket {
    /// Binds a new UDP socket on any avaliable port.
    pub fn new() -> Result<Self> {
        Self::bind("0.0.0.0:0")
    }

    /// Binds a new UDP socket to the specific socket address.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;

        let buffer = [0; BUFFER_SIZE];
        Ok(Socket { socket, buffer })
    }

    pub fn receive<D: DeserializeOwned>(&mut self) -> Result<Option<(SocketAddr, D)>> {
        let (bytes_received, source_addr) = match self.socket.recv_from(&mut self.buffer) {
            Ok(result) => result,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e),
        };

        let mut buf: [u8; 4] = [0, 0, 0, 0];
        buf.copy_from_slice(&self.buffer[0..4]);
        let num = u32::from_be_bytes(buf);
        if num != protocol_id() {
            error!("ignoring pakcet");
            return Err(Error::new(ErrorKind::Other, "oh no!"));
        }

        let data = &self.buffer[4..bytes_received];
        Ok(bincode::deserialize(&data)
            .ok()
            .map(|message| (source_addr, message)))
    }

    pub fn send<A: ToSocketAddrs, S: Serialize>(&mut self, addr: A, message: S) -> Result<()> {
        let mut packet: Vec<u8> = Vec::new();
        let bytes: [u8; 4] = unsafe { transmute(protocol_id().to_be()) };
        packet.extend_from_slice(&bytes);

        let serialized_message = bincode::serialize(&message).unwrap();
        packet.extend_from_slice(&serialized_message);
        self.socket.send_to(&packet, addr)?;

        Ok(())
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
    Command(GridInputEvent),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage<'a> {
    // Use copy-on-write (Cow) here for the grid because we want to
    // borrow the grid from server when it is writing the message, but
    // own the grid when the client receives and deserializes the
    // message. Cow lets us treat borrowed and owned data similarly
    Sync { grid: Cow<'a, Grid> },
}
