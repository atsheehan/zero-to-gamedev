#[macro_use]
extern crate log;
extern crate bincode;
extern crate getopts;

use block_peers::grid::{Grid, GridInputEvent};
use block_peers::logging;
use block_peers::net::{ClientMessage, ServerMessage, Socket};

use getopts::Options;
use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;
const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

const DEFAULT_PORT: u16 = 4485;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

fn main() {
    logging::init();
    let options = get_options();

    let server_addr = SocketAddr::new(DEFAULT_HOST, options.port);
    let mut socket = Socket::bind(server_addr).expect("could not create socket");

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    // Connections holds a list of clients connected, and when there
    // are at least 2 clients connected it will start the game.
    let mut connections = HashSet::<SocketAddr>::new();

    // The running game state: a list of player addrs and a list of
    // grids. I tried to combine the addr and grid into a single
    // struct, but I couldn't figure out how to extract just the grids
    // for the ServerMessage::Sync message.
    let mut game: Option<(Vec<SocketAddr>, Vec<Grid>)> = None;

    'running: loop {
        if game.is_none() && connections.len() >= 2 {
        }

        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            match game {
                Some((ref client_addrs, ref mut grids)) => {
                    for grid in grids.iter_mut() {
                        grid.update();
                    }

                    for (player_id, addr) in client_addrs.iter().enumerate() {
                        let player_id = player_id as u32;
                        let message = ServerMessage::Sync { player_id, grids: Cow::Borrowed(&grids) };
                        socket.send(addr, &message).unwrap();
                    }
                }
                None => {
                    if connections.len() >= 2 {
                        let clients = connections.iter().cloned().take(2).collect();
                        let grids = vec![
                            Grid::new(GRID_HEIGHT, GRID_WIDTH),
                            Grid::new(GRID_HEIGHT, GRID_WIDTH),
                        ];

                        game = Some((clients, grids));
                    }
                }
            }

            previous_instant += tick_duration;
        }

        match socket.receive::<ClientMessage>() {
            Ok(Some((source_addr, ClientMessage::Connect))) => {
                match game {
                    Some(_) => {
                        socket.send(source_addr, &ServerMessage::Reject).unwrap();
                    }
                    None => {
                        connections.insert(source_addr);
                        socket
                            .send(
                                source_addr,
                                ServerMessage::Connected,
                            )
                            .unwrap();
                    }
                }
            }
            Ok(Some((source_addr, ClientMessage::Command { player_id, event }))) => {
                trace!("server received command {:?}", event);

                if let Some((ref client_addrs, ref mut grids)) = game {
                    let player_id = player_id as usize;
                    if player_id < client_addrs.len() && client_addrs[player_id] == source_addr {
                        match event {
                            GridInputEvent::MoveLeft => {
                                grids[player_id].move_piece_left();
                            }
                            GridInputEvent::MoveRight => {
                                grids[player_id].move_piece_right();
                            }
                            GridInputEvent::MoveDown => {
                                grids[player_id].move_piece_down();
                            }
                            GridInputEvent::ForceToBottom => {
                                grids[player_id].move_piece_to_bottom();
                            }
                            GridInputEvent::Rotate => {
                                grids[player_id].rotate();
                            }
                        }
                    }
                }
            }
            Ok(Some((source_addr, ClientMessage::Disconnect))) => {
                trace!("client {} requested to disconnect", source_addr);
                break 'running;
            }
            Ok(None) => {}
            Err(e) => {
                error!("error receiving message: {}", e);
            }
        }
    }
}

struct ServerOptions {
    port: u16,
}

fn get_options() -> ServerOptions {
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

    ServerOptions { port }
}
