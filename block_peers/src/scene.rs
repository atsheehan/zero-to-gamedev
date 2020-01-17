use sdl2::event::Event;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;

pub enum AppLifecycleEvent {
    /// Either the user has requested to quit or some OS event has happened which wants the app to
    /// shutdown.
    Shutdown,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum GameSoundEvent {
    LinesCleared(u8),
    TurnSoundsOff,
    TurnSoundsOn,
    MovePieceDown,
}

pub trait Scene {
    fn lifecycle(
        &mut self,
        _socket: &mut Sender<(SocketAddr, ClientMessage)>,
        _event: AppLifecycleEvent,
    ) {
    }
    fn input(
        self: Box<Self>,
        socket: &mut Sender<(SocketAddr, ClientMessage)>,
        event: Event,
    ) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn handle_message(
        self: Box<Self>,
        socket: &mut Sender<(SocketAddr, ClientMessage)>,
        source_addr: SocketAddr,
        message: ServerMessage,
    ) -> Box<dyn Scene>;
    fn update(
        self: Box<Self>,
        socket: &mut Sender<(SocketAddr, ClientMessage)>,
        sounds: &mut Vec<GameSoundEvent>,
    ) -> Box<dyn Scene>;
    fn should_quit(&self) -> bool {
        false
    }
}
