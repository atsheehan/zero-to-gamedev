use sdl2::event::Event;

use crate::render::Renderer;

pub enum AppLifecycleEvent {
    /// Either the user has requested to quit or some OS event has happened which wants the app to
    /// shutdown.
    Shutdown,
}

pub trait Scene {
    fn lifecycle(&mut self, _event: AppLifecycleEvent) {}
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn update(self: Box<Self>) -> Box<dyn Scene>;
    // fn network() Do we want something like this?
}
