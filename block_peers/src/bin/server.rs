extern crate bincode;
extern crate getopts;

use block_peers::grid::Grid;
use block_peers::net::{ClientMessage, ServerMessage, Socket};

use getopts::Options;
use std::borrow::Cow;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;

const DEFAULT_PORT: u16 = 4485;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt(
        "p",
        "port",
        "bind to the specified port (default 4485)",
        "PORT",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let port: u16 = match matches.opt_get("port") {
        Ok(Some(port)) => port,
        Ok(None) => DEFAULT_PORT,
        Err(_) => panic!("specified port not valid"),
    };

    let server_addr = SocketAddr::new(DEFAULT_HOST, port);
    let mut socket = Socket::bind(server_addr).expect("could not create socket");

    loop {
        match socket.receive::<ClientMessage>() {
            Ok(Some((source_addr, ClientMessage::Connect))) => {
                println!("client at {:?} connected", source_addr);
                let grid = Grid::new(GRID_HEIGHT, GRID_WIDTH);

                socket
                    .send(
                        source_addr,
                        &ServerMessage::Sync {
                            grid: Cow::Borrowed(&grid),
                        },
                    )
                    .unwrap();
            }
            Ok(None) => {}
            Err(_) => {
                println!("something went wrong");
            }
        }
    }
}
