use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::net::SocketAddr;

use crate::grid::{Grid, GridInputEvent, Player};
use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::{AppLifecycleEvent, Scene};

pub struct GameScene {
    players: Vec<Player>,
    socket: Socket,
    address: SocketAddr,
}

impl GameScene {
    pub fn new(players: Vec<Player>, socket: Socket, address: SocketAddr) -> Self {
        Self {
            players,
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
        for player in &self.players {
            player.grid.render(renderer, (player.id - 1) as i32 * 300);
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        match self.socket.receive::<ServerMessage>() {
            Ok(Some((_source_addr, ServerMessage::Sync { players }))) => {
                self.players = players.into_owned();
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
