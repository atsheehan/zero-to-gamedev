use serde::{Deserialize, Serialize};

use crate::grid::Grid;

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
