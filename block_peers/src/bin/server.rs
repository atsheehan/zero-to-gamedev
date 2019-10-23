extern crate bincode;

use std::net::UdpSocket;
use block_peers::net::{ClientMessage, ServerMessage};

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:4485").expect("could not bind to port");

    let mut buffer = [0; 1000];
    let (amount, source_addr) = socket.recv_from(&mut buffer).unwrap();

    let data = &buffer[..amount];

    match bincode::deserialize(&data) {
        Ok(ClientMessage::Connect) => {
            println!("client at {:?} connected", source_addr);
            socket.send_to(&ServerMessage::Ack.into_bytes(), &source_addr).unwrap();
        }
        Err(_) => {
            println!("received unknown message");
        }
    }
}
