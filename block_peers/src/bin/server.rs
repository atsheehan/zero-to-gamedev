#[macro_use]
extern crate log;
extern crate bincode;
extern crate ctrlc;
extern crate getopts;

use block_peers::grid::Grid;
use block_peers::logging;
use block_peers::net::{ClientMessage, ServerMessage, Socket};

use getopts::Options;
use std::borrow::Cow;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;

const DEFAULT_PORT: u16 = 4485;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

fn main() {
    logging::init();

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

    // Shutdown signaling
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_handle = Arc::clone(&should_quit);
    ctrlc::set_handler(move || {
        quit_handle.store(true, Ordering::Relaxed);
    })
    .expect("error setting Ctrl-C handler for graceful shutdown");

    loop {
        if should_quit.load(Ordering::Relaxed) {
            info!("gracefully shutting down server");
            break;
        }

        match socket.receive::<ClientMessage>() {
            Ok(Some((source_addr, ClientMessage::Connect))) => {
                trace!("client at {:?} connected", source_addr);
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
                error!("something went wrong");
            }
        }
    }
}
