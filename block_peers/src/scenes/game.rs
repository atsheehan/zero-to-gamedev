use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::net::SocketAddr;

use crate::grid::{Grid, GridInputEvent};
use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::{AppLifecycleEvent, Scene};

pub struct GameScene {
    grids: Vec<Grid>,
    socket: Socket,
    address: SocketAddr,
}

impl GameScene {
    pub fn new(grids: Vec<Grid>, socket: Socket, address: SocketAddr) -> Self {
        Self {
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
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command { player_id: 0, event: GridInputEvent::MoveLeft },
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
                        &ClientMessage::Command { player_id: 0, event: GridInputEvent::MoveRight },
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
                        &ClientMessage::Command { player_id: 0, event: GridInputEvent::MoveDown },
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
                        &ClientMessage::Command { player_id: 0, event: GridInputEvent::ForceToBottom },
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
                        &ClientMessage::Command { player_id: 0, event: GridInputEvent::Rotate },
                    )
                    .unwrap();
            },
            Event::KeyDown {
                keycode: Some(Keycode::J),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command { player_id: 1, event: GridInputEvent::MoveLeft },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::L),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command { player_id: 1, event: GridInputEvent::MoveRight },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::K),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command { player_id: 1, event: GridInputEvent::MoveDown },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::I),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command { player_id: 1, event: GridInputEvent::ForceToBottom },
                    )
                    .unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::O),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command { player_id: 1, event: GridInputEvent::Rotate },
                    )
                    .unwrap();
            }
            _ => {}
        }
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        let mut idx = 0;
        for grid in &self.grids {
            grid.render(renderer, idx * 300);
            idx += 1;
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        match self.socket.receive::<ServerMessage>() {
            Ok(Some((_source_addr, ServerMessage::Sync { grids }))) => {
                self.grids = grids.into_owned();
                self
            }
            Ok(Some((_source_addr, ServerMessage::Reject))) => {
                error!("received reject message when not appropriate");
                self
            }
            Ok(None) => self,
            Err(_) => {
                error!("received unknown message");
                panic!("expected game state to be given from server on init")
            }
        }
    }
}
