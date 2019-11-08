use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::net::SocketAddr;

use crate::grid::{Grid, GridInputEvent};
use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::{Renderer, VIEWPORT_WIDTH, VIEWPORT_HEIGHT};
use crate::scene::{AppLifecycleEvent, Scene};

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
        let (x_offset, y_offset) = grid_offset(self.grid.size());

        renderer.set_offset(x_offset, y_offset);
        self.grid.render(renderer);
        renderer.set_offset(0, 0);
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

fn grid_offset(grid_size: (u32, u32)) -> (i32, i32) {
    let (grid_width, grid_height) = grid_size;

    ((VIEWPORT_WIDTH - grid_width) as i32 / 2, (VIEWPORT_HEIGHT - grid_height) as i32 / 2)
}
