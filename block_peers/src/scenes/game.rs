use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::net::SocketAddr;

use crate::grid::{Grid, GridInputEvent};
use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::Scene;

pub struct GameScene {
    grid: Grid,
    socket: Socket,
    address: SocketAddr,
}

impl GameScene {
    pub fn new(grid: Grid, socket: Socket, address: SocketAddr) -> Self {
        Self {
            grid,
            socket,
            address,
        }
    }
}

impl Scene for GameScene {
    fn input(mut self: Box<Self>, event: Event) -> Box<dyn Scene> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                self.socket
                    .send(
                        self.address,
                        &ClientMessage::Command(GridInputEvent::MoveLeft),
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
                        &ClientMessage::Command(GridInputEvent::MoveRight),
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
                        &ClientMessage::Command(GridInputEvent::MoveDown),
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
                        &ClientMessage::Command(GridInputEvent::ForceToBottom),
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
                        &ClientMessage::Command(GridInputEvent::Rotate),
                    )
                    .unwrap();
            }
            _ => {}
        }
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        self.grid.render(renderer);
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        /*
        self.grid.update();

        if self.grid.gameover {
            return Box::new(GameOverScene::new());
        }

        self
        */

        match self.socket.receive::<ServerMessage>() {
            Ok(Some((_source_addr, ServerMessage::Sync { grid }))) => {
                self.grid = grid.into_owned();
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
