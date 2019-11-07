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

struct Player {
    grid: Grid,
    addr: SocketAddr,
}

fn main() {
    logging::init();
    let options = get_options();

    let server_addr = SocketAddr::new(DEFAULT_HOST, options.port);
    let mut socket = Socket::bind(server_addr).expect("could not create socket");

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    let mut connection: Option<(SocketAddr, Vec<Grid>)> = None;
    let mut connections = HashSet::<SocketAddr>::new();
    let mut game: Option<Vec<Player>> = None;

    'running: loop {
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            if let Some((source_addr, ref mut grids)) = connection {
                for grid in grids.iter_mut() {
                    grid.update();
                }

                socket
                    .send(
                        source_addr,
                        &ServerMessage::Sync {
                            grids: Cow::Borrowed(&grids),
                        },
                    )
                    .unwrap();
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

            //     if connection.is_none() {
            //         debug!("client at {:?} connected", source_addr);
            //         let grids = vec![
            //             Grid::new(GRID_HEIGHT, GRID_WIDTH),
            //             Grid::new(GRID_HEIGHT, GRID_WIDTH),
            //         ];

            //         socket
            //             .send(
            //                 source_addr,
            //                 &ServerMessage::Connected {
            //                     player_id: 0,
            //                 },
            //             )
            //             .unwrap();

            //         socket
            //             .send(
            //                 source_addr,
            //                 &ServerMessage::Sync {
            //                     grids: Cow::Borrowed(&grids),
            //                 },
            //             )
            //             .unwrap();

            //         connection = Some((source_addr, grids));
            //     } else {
            //         debug!(
            //             "rejecting client {} since a game is already in progress",
            //             source_addr
            //         );
            //         socket.send(source_addr, &ServerMessage::Reject).unwrap();
            //     }
            // }
            Ok(Some((_source_addr, ClientMessage::Command { player_id, event }))) => {
                trace!("server received command {:?}", event);

                // if let Some((_, ref mut grids)) = connection {
                //     match event {
                //         GridInputEvent::MoveLeft => {
                //             grids[player_id as usize].move_piece_left();
                //         }
                //         GridInputEvent::MoveRight => {
                //             grids[player_id as usize].move_piece_right();
                //         }
                //         GridInputEvent::MoveDown => {
                //             grids[player_id as usize].move_piece_down();
                //         }
                //         GridInputEvent::ForceToBottom => {
                //             grids[player_id as usize].move_piece_to_bottom();
                //         }
                //         GridInputEvent::Rotate => {
                //             grids[player_id as usize].rotate();
                //         }
                //     }
                // }
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
