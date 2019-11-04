#[macro_use]
extern crate log;
extern crate bincode;
extern crate getopts;

use block_peers::grid::{Grid, GridInputEvent, Player};
use block_peers::logging;
use block_peers::net::{ClientMessage, ServerMessage, Socket};

use getopts::Options;
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

fn main() {
    logging::init();
    let options = get_options();

    let server_addr = SocketAddr::new(DEFAULT_HOST, options.port);
    let mut socket = Socket::bind(server_addr).expect("could not create socket");

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    let mut connection: Option<(SocketAddr, Player)> = None;

    'running: loop {
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            if let Some((source_addr, ref mut player)) = connection {
                player.grid.update();
                socket
                    .send(
                        source_addr,
                        &ServerMessage::Sync {
                            player: Cow::Borrowed(&player),
                        },
                    )
                    .unwrap();
            }

            previous_instant += tick_duration;
        }

        match socket.receive::<ClientMessage>() {
            Ok(Some((source_addr, ClientMessage::Connect))) => {
                if connection.is_none() {
                    debug!("client at {:?} connected", source_addr);
                    let player = Player { id: 1, grid: Grid::new(GRID_HEIGHT, GRID_WIDTH) };

                    socket
                        .send(
                            source_addr,
                            &ServerMessage::Sync {
                                player: Cow::Borrowed(&player),
                            },
                        )
                        .unwrap();

                    connection = Some((source_addr, player));
                } else {
                    debug!(
                        "rejecting client {} since a game is already in progress",
                        source_addr
                    );
                    socket.send(source_addr, &ServerMessage::Reject).unwrap();
                }
            }
            Ok(Some((_source_addr, ClientMessage::Command(command)))) => {
                trace!("server received command {:?}", command);

                if let Some((_, ref mut player)) = connection {
                    match command {
                        GridInputEvent::MoveLeft => {
                            player.grid.move_piece_left();
                        }
                        GridInputEvent::MoveRight => {
                            player.grid.move_piece_right();
                        }
                        GridInputEvent::MoveDown => {
                            player.grid.move_piece_down();
                        }
                        GridInputEvent::ForceToBottom => {
                            player.grid.move_piece_to_bottom();
                        }
                        GridInputEvent::Rotate => {
                            player.grid.rotate();
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
