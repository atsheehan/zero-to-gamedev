use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::grid::Grid;
use crate::render::Renderer;

pub trait Scene {
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn update(&mut self) -> Option<Box<dyn Scene>>;
}

// ------------
// Title Scene
// ------------
pub struct TitleScene {
    grid: Grid,
}

impl TitleScene {
    pub fn new(grid: Grid) -> Self {
        Self { grid }
    }
}

impl Scene for TitleScene {
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => Box::new(GameScene::new(self.grid)),

            _ => self,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.fill_rect(Rect::new(0, 0, 200, 200), Color::RGB(255, 0, 0));
    }

    fn update(&mut self) -> Option<Box<dyn Scene>> {
        None
    }
}

// ------------
// Game Scene
// ------------
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

    fn update(&mut self) -> Option<Box<dyn Scene>> {
        self.grid.update();

        None
    }
}

// ---------------
// Game Over Scene
// ---------------
pub struct GameOverScene {}

impl GameOverScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for GameOverScene {
    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        // TODO: render game over screen
    }

    fn update(&mut self) -> Option<Box<dyn Scene>> {
        None
    }
}
