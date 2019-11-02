use sdl2::event::Event;

use crate::render::Renderer;

pub enum AppLifecycleEvent {
    Quit,
    EnteredBackground,
    EnteredForeground,
}

pub trait Scene {
    fn lifecycle(&mut self, event: AppLifecycleEvent);
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn update(self: Box<Self>) -> Box<dyn Scene>;
    // fn network() Do we want something like this?
}
