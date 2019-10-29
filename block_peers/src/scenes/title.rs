use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::net::SocketAddr;

use crate::ai::DumbAI;
use crate::grid::Grid;
use crate::piece::Piece;
use crate::render::{Renderer, WindowSize};
use crate::scene::Scene;
use crate::scenes::ConnectScene;

pub struct TitleScene {
    server_addr: SocketAddr,
    ai: DumbAI,
}

impl TitleScene {
    pub fn new(server_addr: SocketAddr, size: WindowSize) -> Self {
        let width = size.width / 20;
        let height = size.height / 20;
        let mut background_grid = Grid::new(height, width);

        // Set some pieces on the board
        background_grid.place_piece_at_bottom(Piece::new(6).move_left());
        background_grid.place_piece_at_bottom(Piece::new(0).rotate().rotate().move_right_times(2));
        background_grid.place_piece_at_bottom(Piece::new(2));
        background_grid.place_piece_at_bottom(Piece::new(2).move_right_times(4));
        background_grid.place_piece_at_bottom(Piece::new(4).move_right_times(2));
        background_grid.place_piece_at_bottom(Piece::new(1).rotate().move_right_times(8));

        Self {
            server_addr,
            ai: DumbAI::new(background_grid),
        }
    }
}

impl Scene for TitleScene {
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => Box::new(ConnectScene::new(self.server_addr)),

            _ => self,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        self.ai.render(renderer);
        renderer.render_title(180, 200);
        renderer.render_space(350, 320);
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        self.ai.update();
        self
    }
}
