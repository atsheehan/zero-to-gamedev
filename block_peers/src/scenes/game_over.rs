use sdl2::event::Event;

use std::net::SocketAddr;

use crate::net::{ClientMessage, Socket};
use crate::render::Renderer;
use crate::scene::{AppLifecycleEvent, Scene};
use crate::text::Text;

pub struct GameOverScene {
    socket: Socket,
    address: SocketAddr,
}

impl GameOverScene {
    pub fn new(socket: Socket, address: SocketAddr) -> Self {
        Self { socket, address }
    }
}

impl Scene for GameOverScene {
    fn lifecycle(&mut self, event: AppLifecycleEvent) {
        match event {
            AppLifecycleEvent::Shutdown => {
                trace!("sending disconnect to the server");
                self.socket
                    .send(self.address, &ClientMessage::Disconnect)
                    .unwrap();
            }
        }
    }

    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
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

    fn update(self: Box<Self>) -> Box<dyn Scene> {
        self
    }
}
