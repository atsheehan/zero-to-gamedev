use sdl2::event::Event;

use crate::render::Renderer;

pub trait Scene {
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn update(&mut self) -> Option<Box<dyn Scene>>;
    // fn network() Do we want something like this?
}
