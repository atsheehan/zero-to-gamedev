use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use super::game::GameScene;
use crate::grid::Grid;
use crate::render::Renderer;
use crate::scene::Scene;

pub struct TitleScene {
    server_state: Grid,
    background_grid: Grid,
}

impl TitleScene {
    pub fn new(grid: Grid) -> Self {
        Self {
            server_state: grid,
            background_grid: Grid::new(40, 40),
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
        self.background_grid.render(renderer);
    }

    fn update(&mut self) -> Option<Box<dyn Scene>> {
        None
    }
}
