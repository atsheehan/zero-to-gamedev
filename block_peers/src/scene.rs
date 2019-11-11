use sdl2::event::Event;
use std::net::SocketAddr;

use crate::net::{ServerMessage, Socket};
use crate::render::Renderer;

pub enum AppLifecycleEvent {
    /// Either the user has requested to quit or some OS event has happened which wants the app to
    /// shutdown.
    Shutdown,
}

pub trait Scene {
    fn lifecycle(&mut self, _socket: &mut Socket, _event: AppLifecycleEvent) {}
    fn input(self: Box<Self>, socket: &mut Socket, event: Event) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn handle_message(
        self: Box<Self>,
        socket: &mut Socket,
        source_addr: SocketAddr,
        message: ServerMessage,
    ) -> Box<dyn Scene>;
    fn update(self: Box<Self>, socket: &mut Socket) -> Box<dyn Scene>;
    fn should_quit(&self) -> bool {
        false
    }
}
