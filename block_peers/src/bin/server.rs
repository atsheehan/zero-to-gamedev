#[macro_use]
extern crate log;
extern crate bincode;
extern crate getopts;

use block_peers::grid::{Grid, GridInputEvent};
use block_peers::logging;
use block_peers::net::{ClientMessage, ServerMessage, Socket};

use getopts::Options;
use rand::Rng;
use std::borrow::Cow;
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

const MAX_CLIENTS: usize = 1;

struct Client {
    address: SocketAddr,
    challenge_confirmed: bool,
    salt: u64,
    // port?
    // state?
    // grid: Grid,
}

impl Client {
    pub fn new(address: SocketAddr) -> Self {
        let mut rng = rand::thread_rng();
        let salt = rng.gen_range(0, std::u64::MAX);

        // generate salt
        // create new grid
        // challenge confirmed no
        Self {
            address,
            salt,
            challenge_confirmed: false,
        }
    }
}

fn main() {
    logging::init();
    let options = get_options();

    let server_addr = SocketAddr::new(DEFAULT_HOST, options.port);
    let mut socket = Socket::bind(server_addr).expect("could not create socket");

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    let mut player: Option<(SocketAddr, Grid)> = None;
    let mut connections: Vec<Client> = Vec::new();

    'running: loop {
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            if let Some((source_addr, ref mut grid)) = player {
                grid.update();
                socket
                    .send(
                        source_addr,
                        &ServerMessage::Sync {
                            grid: Cow::Borrowed(grid),
                        },
                    )
                    .unwrap();
            }

            // TODO: for each confirmed client, update the grid and send sync message

            previous_instant += tick_duration;
        }

        match socket.receive::<ClientMessage>() {
            Ok(Some((source_addr, ClientMessage::Connect))) => {
                let mut found_client = false;
                for client in connections.iter() {
                    if client.address == source_addr {
                        debug!("client already awaiting challenge");

                        if client.challenge_confirmed {
                            found_client = true;
                        } else {
                            socket
                                .send(source_addr, &ServerMessage::Challenge { salt: client.salt })
                                .unwrap();
                            found_client = true;
                        }
                    }
                }

                if !found_client {
                    let client = Client::new(source_addr);
                    let salt = client.salt.clone();
                    connections.push(client);
                    socket
                        .send(source_addr, &ServerMessage::Challenge { salt })
                        .unwrap();
                }
            }
            Ok(Some((source_addr, ClientMessage::ChallengeResponse { salt }))) => {
                debug!("received challenge response {}", salt);

                for client in connections.iter_mut() {
                    if client.address == source_addr && client.salt == salt {
                        client.challenge_confirmed = true;
                        socket
                            .send(source_addr, &ServerMessage::ConnectionAccepted)
                            .unwrap();

                        if player.is_none() {
                            debug!("client at {:?} connected", source_addr);
                            let grid = Grid::new(GRID_HEIGHT, GRID_WIDTH);

                            socket
                                .send(
                                    source_addr,
                                    &ServerMessage::Sync {
                                        grid: Cow::Borrowed(&grid),
                                    },
                                )
                                .unwrap();

                            player = Some((source_addr, grid));
                        } else {
                            debug!(
                                "rejecting client {} since a game is already in progress",
                                source_addr
                            );
                            socket
                                .send(source_addr, &ServerMessage::ConnectionRejected)
                                .unwrap();
                        }
                    }
                }
            }
            Ok(Some((_source_addr, ClientMessage::Command(command)))) => {
                trace!("server received command {:?}", command);
                // TODO: check coming from a confirmed client

                if let Some((_, ref mut grid)) = player {
                    match command {
                        GridInputEvent::MoveLeft => {
                            grid.move_piece_left();
                        }
                        GridInputEvent::MoveRight => {
                            grid.move_piece_right();
                        }
                        GridInputEvent::MoveDown => {
                            grid.move_piece_down();
                        }
                        GridInputEvent::ForceToBottom => {
                            grid.move_piece_to_bottom();
                        }
                        GridInputEvent::Rotate => {
                            grid.rotate();
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
