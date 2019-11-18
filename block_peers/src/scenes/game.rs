use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use std::net::SocketAddr;

use crate::brick::CELL_SIZE;
use crate::grid::{Grid, GridInputEvent};
use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::{Image, Opacity, Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::scene::{AppLifecycleEvent, Scene};
use crate::scenes::GameOverScene;

pub struct GameScene {
    player_id: u32,
    grids: Vec<Grid>,
    address: SocketAddr,
}

impl GameScene {
    pub fn new(player_id: u32, grids: Vec<Grid>, address: SocketAddr) -> Self {
        Self {
            player_id,
            grids,
            address,
        }
    }
}

impl Scene for GameScene {
    fn lifecycle(&mut self, socket: &mut Socket, event: AppLifecycleEvent) {
        match event {
            AppLifecycleEvent::Shutdown => {
                trace!("sending disconnect to the server");
                socket
                    .send(self.address, &ClientMessage::Disconnect)
                    .unwrap();
            }
        }
    }

    fn input(self: Box<Self>, socket: &mut Socket, event: Event) -> Box<dyn Scene> {
        let player_id = self.player_id;

        match event {
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                socket
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
                socket
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
                socket
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
                socket
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
                socket
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
        renderer.render_image(
            Image::PlayingField,
            Rect::new(0, 0, 800, 600),
            Opacity::Opaque,
        );

        for (idx, grid) in self.grids.iter().enumerate() {
            let (x_offset, y_offset) =
                grid_offset(grid.size(), idx as u32, self.grids.len() as u32);
            renderer.with_relative_offset(x_offset, y_offset, |renderer| grid.render(renderer));
        }
    }

    fn update(self: Box<Self>, _socket: &mut Socket) -> Box<dyn Scene> {
        if self.grids.iter().any(|grid| grid.gameover) {
            Box::new(GameOverScene::new(self.address))
        } else {
            self
        }
    }

    fn handle_message(
        mut self: Box<Self>,
        _socket: &mut Socket,
        _source_addr: SocketAddr,
        message: ServerMessage,
    ) -> Box<dyn Scene> {
        match message {
            ServerMessage::Sync { grids, .. } => {
                self.grids = grids.into_owned();
                self
            }
            _ => self,
        }
    }
}

fn grid_offset(grid_size: (u32, u32), index: u32, num_grids: u32) -> (i32, i32) {
    let (grid_width, grid_height) = grid_size;

    // Add height offset for staged piece and score
    let staged_height = 3 * CELL_SIZE;
    // Add height for the margin between each grid section (8 * 2)
    let section_margin = 16;

    // Slice up the viewport into equal sized chunks
    let chunk_width = VIEWPORT_WIDTH / num_grids;

    // Center the grid within the chunk
    (
        (index * chunk_width) as i32 + (chunk_width - grid_width) as i32 / 2,
        (VIEWPORT_HEIGHT - (grid_height - staged_height + section_margin)) as i32 / 2,
    )
}
