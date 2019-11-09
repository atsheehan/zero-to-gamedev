use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::net::SocketAddr;

use crate::grid::{Grid, GridInputEvent};
use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::{Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::scene::{AppLifecycleEvent, Scene};

pub struct GameScene {
    player_id: u32,
    grids: Vec<Grid>,
    socket: Socket,
    address: SocketAddr,
}

impl GameScene {
    pub fn new(player_id: u32, grids: Vec<Grid>, socket: Socket, address: SocketAddr) -> Self {
        Self {
            player_id,
            grids,
            socket,
            address,
        }
    }
}

impl Scene for GameScene {
    fn lifecycle(&mut self, event: AppLifecycleEvent) {
        match event {
            AppLifecycleEvent::Shutdown => {
                trace!("sending disconnect to the server");
                self.socket
                    .send(self.address, &ClientMessage::Disconnect)
                    .unwrap();
            }
        }
    }

    fn input(mut self: Box<Self>, event: Event) -> Box<dyn Scene> {
        let player_id = self.player_id;

        match event {
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command {
                            player_id,
                            event: GridInputEvent::MoveLeft,
                        },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command {
                            player_id,
                            event: GridInputEvent::MoveRight,
                        },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command {
                            player_id,
                            event: GridInputEvent::MoveDown,
                        },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command {
                            player_id,
                            event: GridInputEvent::ForceToBottom,
                        },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::E),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command {
                            player_id,
                            event: GridInputEvent::Rotate,
                        },
                    )
                    .unwrap();
            }
            _ => {}
        }
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        for (idx, grid) in self.grids.iter().enumerate() {
            let (x_offset, y_offset) =
                grid_offset(grid.size(), idx as u32, self.grids.len() as u32);
            renderer.set_offset(x_offset, y_offset);
            grid.render(renderer);
            renderer.set_offset(0, 0);
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        match self.socket.receive::<ServerMessage>() {
            Ok(Some((
                _source_addr,
                ServerMessage::Sync {
                    player_id: _,
                    grids,
                },
            ))) => {
                self.grids = grids.into_owned();
                self
            }
            Ok(Some((_source_addr, ServerMessage::Reject))) => {
                error!("received reject message when not appropriate");
                self
            }
            Ok(None) => self,
            Ok(Some((_source_addr, message))) => {
                debug!("received unexpected message: {:?}", message);
                self
            }
            Err(_) => {
                error!("received unknown message");
                panic!("expected game state to be given from server on init")
            }
        }
    }
}

fn grid_offset(grid_size: (u32, u32), index: u32, num_grids: u32) -> (i32, i32) {
    let (grid_width, grid_height) = grid_size;

    // Slice up the viewport into equal sized chunks
    let chunk_width = VIEWPORT_WIDTH / num_grids;

    // Center the grid within the chunk
    (
        (index * chunk_width) as i32 + (chunk_width - grid_width) as i32 / 2,
        (VIEWPORT_HEIGHT - grid_height) as i32 / 2,
    )
}
