extern crate bincode;

use block_peers::grid::Grid;
use block_peers::net::{ClientMessage, ServerMessage};

use std::net::UdpSocket;

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:4485").expect("could not bind to port");

    let mut buffer = [0; 1000];
    let (amount, source_addr) = socket.recv_from(&mut buffer).unwrap();

    let data = &buffer[..amount];

    let grid = Grid::new(GRID_HEIGHT, GRID_WIDTH);

    match bincode::deserialize(&data) {
        Ok(ClientMessage::Connect) => {
            println!("client at {:?} connected", source_addr);
            socket
                .send_to(&ServerMessage::Ack { grid }.into_bytes(), &source_addr)
                .unwrap();
        }
        Err(_) => {
            println!("received unknown message");
        }
    }
}
