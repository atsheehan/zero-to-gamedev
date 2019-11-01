use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::constants::PROTOCOL_VERSION;
use crate::grid::{Grid, GridInputEvent};

lazy_static! {
    static ref PROTOCOL_ID: u32 = {
        let mut total: u32 = 0;
        for b in "Block Wars".bytes() {
            total += b as u32;
        }
        total += PROTOCOL_VERSION;
        total
    };
}

const BUFFER_SIZE: usize = 4096;

// ---------------------------
// Server <--> Client Messages
// ---------------------------

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

// -------
// Socket
// -------

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

        let data = &self.buffer[..bytes_received];
        let packet: Packet =
            bincode::deserialize(&data).expect("this shouldn't panic like this :(");

        if packet.header.protocol_id != *PROTOCOL_ID {
            error!("unknown incoming packet, ignoring");
            return Err(Error::new(ErrorKind::Other, "Unknown packet type"));
        }

        Ok(bincode::deserialize(&packet.body)
            .ok()
            .map(|message| (source_addr, message)))
    }

    pub fn send<A: ToSocketAddrs, S: Serialize>(&mut self, addr: A, message: S) -> Result<()> {
        let bytes = Packet::new(message).as_bytes();
        self.socket.send_to(&bytes, addr)?;
        Ok(())
    }
}

// -------
// Packet
// -------

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
struct PacketHeader {
    protocol_id: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Packet {
    header: PacketHeader,
    body: Vec<u8>,
}

impl Packet {
    fn new<S: Serialize>(message: S) -> Self {
        let body = bincode::serialize(&message).unwrap();

        Self {
            header: PacketHeader {
                protocol_id: *PROTOCOL_ID,
            },
            body,
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("error serializing packet into bytes for transmission")
    }
}
