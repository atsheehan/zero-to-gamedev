use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
}

impl ClientMessage {
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Ack,
}

impl ServerMessage {
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
