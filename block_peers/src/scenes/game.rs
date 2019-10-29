use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use super::game_over::GameOverScene;
use crate::grid::Grid;
use crate::render::Renderer;
use crate::scene::Scene;

pub struct GameScene {
    grid: Grid,
}

impl GameScene {
    pub fn new(grid: Grid) -> Self {
        Self { grid }
    }
}

impl Scene for GameScene {
    fn input(mut self: Box<Self>, event: Event) -> Box<dyn Scene> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                self.grid.move_piece_left();
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                self.grid.move_piece_right();
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                self.grid.move_piece_down();
            }
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                self.grid.move_piece_to_bottom();
            }
            Event::KeyDown {
                keycode: Some(Keycode::E),
                ..
            } => {
                self.grid.rotate();
            }
            _ => {}
        }
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        self.grid.render(renderer);
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        self.grid.update();

        if self.grid.gameover {
            return Box::new(GameOverScene::new());
        }

        self
    }
}
