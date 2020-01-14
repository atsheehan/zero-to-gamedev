#[macro_use]
extern crate log;
extern crate async_std;
extern crate bincode;
extern crate getopts;

use block_peers::grid::{Grid, GridAttackEvent, GridInputEvent};
use block_peers::logging;
use block_peers::net::{ClientMessage, ServerEvent, ServerMessage, ServerSocket};

use async_std::net::SocketAddr;
use async_std::task;

use getopts::Options;
use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;
const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

const DEFAULT_PORT: u16 = 4485;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
const DEFAULT_PLAYERS_PER_GAME: u32 = 1;

fn main() {
    logging::init();
    let options = get_options();

    let server_addr = SocketAddr::new(DEFAULT_HOST, options.port);
    let mut socket = task::block_on(async { ServerSocket::bind(server_addr).await })
        .expect("could not create socket");

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    // The running game state: a list of player addrs and a list of
    // grids. I tried to combine the addr and grid into a single
    // struct, but I couldn't figure out how to extract just the grids
    // for the ServerMessage::Sync message.
    //
    // The player_id is the index into each list (so it should be
    // either 0 or 1 for a 2-player game).
    let mut game: Option<(Vec<SocketAddr>, Vec<Grid>)> = None;

    // Holds a list of clients connected, and when there are at least
    // 2 clients connected it will start the game.
    let mut connected_clients: HashSet<SocketAddr> = HashSet::new();

    'running: loop {
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            match game {
                // If there is an active game, update the grids each
                // tick and sync the state to each client.
                Some((ref client_addrs, ref mut grids)) => {
                    let mut attacks: Vec<(u32, GridAttackEvent)> = Vec::new();
                    for (i, grid) in grids.iter_mut().enumerate() {
                        if let Some(attack) = grid.update() {
                            attacks.push((i as u32, attack));
                        }
                    }

                    for (from_player_id, attack) in attacks.iter() {
                        for (i, grid) in grids.iter_mut().enumerate() {
                            if i != *from_player_id as usize {
                                grid.attack(attack.clone());
                            }
                        }
                    }

                    for (player_id, addr) in client_addrs.iter().enumerate() {
                        let player_id = player_id as u32;
                        let message = ServerMessage::Sync {
                            player_id,
                            grids: Cow::Borrowed(&grids),
                        };
                        task::block_on(async { socket.send(addr, &message).await }).unwrap();
                    }

                    for grid in grids.iter_mut() {
                        grid.sound_events.clear();
                    }
                }
                // If there isn't an active game, check if we have at
                // least two clients connected and create a new game
                // state. The next tick will send the game state to
                // the clients.
                None => {
                    let players_per_game = options.players_per_game as usize;

                    if connected_clients.len() >= players_per_game {
                        let clients = connected_clients
                            .iter()
                            .cloned()
                            .take(players_per_game)
                            .collect();
                        let mut grids = Vec::with_capacity(players_per_game);
                        for _ in 0..players_per_game {
                            grids.push(Grid::new(GRID_HEIGHT, GRID_WIDTH));
                        }

                        game = Some((clients, grids));
                    }
                }
            }

            previous_instant += tick_duration;
        }

        match task::block_on(async { socket.receive().await }) {
            Ok(Some(ServerEvent::ClientConnected(addr))) => {
                connected_clients.insert(addr);
            }
            Ok(Some(ServerEvent::ClientDisconnected(_addr))) => {
                break 'running;
            }
            Ok(Some(ServerEvent::GameEvent(addr, message))) => {
                match message {
                    ClientMessage::Command { player_id, event } => {
                        trace!("server received command {:?}", event);

                        // If there's an active game...
                        if let Some((ref client_addrs, ref mut grids)) = game {
                            let player_id = player_id as usize;

                            // Check the specified player_id lines up with the
                            // source addr before taking any action
                            if player_id < client_addrs.len() && client_addrs[player_id] == addr {
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
                    _ => {}
                }
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
    players_per_game: u32,
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

    opts.optopt(
        "n",
        "num-players",
        "numbers of players per game (default 1)",
        "COUNT",
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

    let players_per_game: u32 = match matches.opt_get("num-players") {
        Ok(Some(count)) => count,
        Ok(None) => DEFAULT_PLAYERS_PER_GAME,
        Err(_) => panic!("specified num-players is not valid"),
    };

    ServerOptions {
        port,
        players_per_game,
    }
}
