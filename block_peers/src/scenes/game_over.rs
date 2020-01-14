use sdl2::event::Event;

use async_std::task;
use std::net::SocketAddr;

use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::{AppLifecycleEvent, GameSoundEvent, Scene};
use crate::text::Text;

pub struct GameOverScene {
    address: SocketAddr,
}

impl GameOverScene {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }
}

impl Scene for GameOverScene {
    fn lifecycle(&mut self, socket: &mut Socket, event: AppLifecycleEvent) {
        match event {
            AppLifecycleEvent::Shutdown => {
                trace!("sending disconnect to the server");

                task::block_on(async {
                    socket.send(self.address, &ClientMessage::Disconnect).await
                })
                .unwrap();
            }
        }
    }

    fn input(self: Box<Self>, _socket: &mut Socket, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.render_text(
            Text::new("Game Over")
                .center_xy(400, 300)
                .height(40)
                .build(),
        );
    }

    fn handle_message(
        self: Box<Self>,
        _socket: &mut Socket,
        _source_addr: SocketAddr,
        _message: ServerMessage,
    ) -> Box<dyn Scene> {
        self
    }

    fn update(
        self: Box<Self>,
        _socket: &mut Socket,
        _sounds: &mut Vec<GameSoundEvent>,
    ) -> Box<dyn Scene> {
        self
    }
}
