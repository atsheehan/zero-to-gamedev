use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use super::game::GameScene;
use crate::ai::DumbAI;
use crate::grid::Grid;
use crate::piece::Piece;
use crate::render::{Renderer, WindowSize};
use crate::scene::Scene;

pub struct TitleScene {
    server_state: Grid,
    ai: DumbAI,
}

impl TitleScene {
    pub fn new(grid: Grid, size: WindowSize) -> Self {
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
            server_state: grid,
            ai: DumbAI::new(background_grid),
        }
    }
}

impl Scene for TitleScene {
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => Box::new(GameScene::new(self.server_state)),

            _ => self,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        self.ai.render(renderer);
    }

    fn update(&mut self) -> Option<Box<dyn Scene>> {
        self.ai.update();

        None
    }
}
