use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::render::Renderer;
use crate::scene::{AppLifecycleEvent, Scene};

pub struct GameOverScene {}

impl GameOverScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for GameOverScene {
    fn lifecycle(&mut self, _event: AppLifecycleEvent) {}

    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.fill_rect(Rect::new(0, 0, 200, 200), Color::RGB(0, 255, 0));
    }

    fn update(self: Box<Self>) -> Box<dyn Scene> {
        self
    }
}
