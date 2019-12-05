use rand::Rng;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::codec::{gzip_decode, gzip_encode};
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

/// Buffer size used for incoming packets. We don't currently have fragmentation so if a packet
/// exceeds this size undefined behavior will ensue.
const BUFFER_SIZE: usize = 4096;
/// Block Wars game protocol version used in determining if incoming packets are allowed or
/// should be ignored.
const PROTOCOL_VERSION: u32 = 1;

// ---------------------------
// Server <--> Client Messages
// ---------------------------

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
    Command {
        player_id: u32,
        event: GridInputEvent,
    },
    Disconnect,
    ChallengeResponse {
        salt: u64,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage<'a> {
    // Use copy-on-write (Cow) here for the grid because we want to
    // borrow the grid from server when it is writing the message, but
    // own the grid when the client receives and deserializes the
    // message. Cow lets us treat borrowed and owned data similarly
    Sync {
        player_id: u32,
        grids: Cow<'a, Vec<Grid>>,
    },
    // Server can't accept anymore incoming connections
    ConnectionRejected,
    // Client challenge was successful and connection has been accepted
    ConnectionAccepted,
    // Server needs to confirm that the client is who they say they are
    // by sending a unique salt and waiting for the client to respond
    // with the salt.
    Challenge {
        salt: u64,
    },
}

pub enum ServerEvent {
    ClientConnected(SocketAddr),
    ClientDisconnected(SocketAddr),
    GameEvent(SocketAddr, ClientMessage),
}

pub struct ServerSocket {
    socket: Socket,
    connections: HashMap<SocketAddr, Connection>,
}

impl ServerSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let socket = Socket::bind(addr)?;
        let connections = HashMap::new();
        Ok(Self {
            socket,
            connections,
        })
    }

    pub fn receive(&mut self) -> Result<Option<ServerEvent>> {
        match self.socket.receive::<ClientMessage>() {
            Ok(Some((source_addr, ClientMessage::Connect))) => {
                match self.connections.entry(source_addr) {
                    Entry::Vacant(entry) => {
                        let client = Connection::new(source_addr);
                        let salt = client.salt;
                        entry.insert(client);
                        self.socket
                            .send(source_addr, &ServerMessage::Challenge { salt })
                            .unwrap();
                    }
                    Entry::Occupied(entry) => {
                        self.socket
                            .send(
                                source_addr,
                                &ServerMessage::Challenge {
                                    salt: entry.get().salt,
                                },
                            )
                            .unwrap();
                    }
                }
                Ok(None)
            }
            Ok(Some((source_addr, ClientMessage::ChallengeResponse { salt }))) => {
                match self.connections.entry(source_addr) {
                    Entry::Vacant(_) => Ok(None),
                    Entry::Occupied(mut entry) => {
                        if entry.get().salt == salt {
                            entry.get_mut().challenge_confirmed = true;
                            self.socket
                                .send(source_addr, &ServerMessage::ConnectionAccepted)
                                .unwrap();
                        }
                        Ok(Some(ServerEvent::ClientConnected(source_addr)))
                    }
                }
            }
            Ok(Some((source_addr, ClientMessage::Disconnect))) => {
                // TODO: check that they exist in the connections
                Ok(Some(ServerEvent::ClientDisconnected(source_addr)))
            }
            Ok(Some((source_addr, client_message))) => {
                Ok(Some(ServerEvent::GameEvent(source_addr, client_message)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn send<A: ToSocketAddrs>(&mut self, addr: A, message: &ServerMessage) -> Result<()> {
        self.socket.send(addr, message)
    }
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
        let decoded = gzip_decode(&data);

        match bincode::deserialize::<Packet<D>>(&decoded) {
            Ok(packet) => {
                if packet.header.protocol_id != *PROTOCOL_ID {
                    return Err(Error::new(ErrorKind::Other, "incorrect protocol id"));
                }

                Ok(Some((source_addr, packet.message)))
            }
            Err(_) => Err(Error::new(ErrorKind::Other, "unable to deserialize packet")),
        }
    }

    pub fn send<A: ToSocketAddrs, S: Serialize>(&mut self, addr: A, message: S) -> Result<()> {
        let packet = Packet::new(message);
        let bytes = gzip_encode(&packet.as_bytes());
        self.socket.send_to(&bytes, addr)?;
        Ok(())
    }
}

struct Connection {
    // TODO: reassess whether we need when introducing multiplayer
    _address: SocketAddr,
    challenge_confirmed: bool,
    salt: u64,
}

impl Connection {
    fn new(address: SocketAddr) -> Self {
        let mut rng = rand::thread_rng();
        let salt = rng.gen_range(0, std::u64::MAX);

        Self {
            _address: address,
            salt,
            challenge_confirmed: false,
        }
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
struct Packet<S> {
    header: PacketHeader,
    message: S,
}

impl<S: Serialize> Packet<S> {
    fn new(message: S) -> Self {
        Self {
            header: PacketHeader {
                protocol_id: *PROTOCOL_ID,
            },
            message,
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("error serializing packet into bytes for transmission")
    }
}
