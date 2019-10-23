extern crate bincode;
extern crate getopts;

use block_peers::grid::Grid;
use block_peers::net::{ClientMessage, ServerMessage};

use getopts::Options;
use std::env;
use std::net::UdpSocket;

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;

const DEFAULT_PORT: u16 = 4485;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("p", "port", "bind to the specified port (default 4485)", "PORT");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!(f.to_string()) }
    };

    let port: u16 = match matches.opt_get("port") {
        Ok(Some(port)) => { port }
        Ok(None) => { DEFAULT_PORT }
        Err(_) => { panic!("specified port not valid") }
    };

    let socket = UdpSocket::bind(("0.0.0.0", port)).expect("could not bind to port");

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
