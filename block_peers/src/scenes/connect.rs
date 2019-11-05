use sdl2::event::Event;
use std::net::SocketAddr;

use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::Scene;
use crate::scenes::GameScene;
use crate::text::Text;

pub struct ConnectScene {
    server_addr: SocketAddr,
    socket: Socket,
    was_rejected: bool,
}

impl ConnectScene {
    pub fn new(server_addr: SocketAddr) -> Self {
        let socket = Socket::new().expect("could not open a new socket");

        Self {
            server_addr,
            socket,
            was_rejected: false,
        }
    }
}

impl Scene for ConnectScene {
    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        if self.was_rejected {
            renderer.render_text(Text::new("REJECTED").center_xy(400, 300).height(40).build());
        } else {
            renderer.render_text(
                Text::new("Connecting to server...")
                    .center_xy(400, 300)
                    .height(40)
                    .build(),
            );
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        if self.was_rejected {
            return self;
        }

        self.socket
            .send(self.server_addr, &ClientMessage::Connect)
            .unwrap();

        match self.socket.receive::<ServerMessage>() {
            Ok(Some((source_addr, ServerMessage::Sync { grids }))) => {
                debug!("connected to server at {:?}", source_addr);
                Box::new(GameScene::new(
                    grids.into_owned(),
                    self.socket,
                    self.server_addr,
                ))
            }
            Ok(Some((source_addr, ServerMessage::Reject))) => {
                error!("client {} was rejected!", source_addr);
                self.was_rejected = true;
                self
            }
            Ok(None) => self,
            Err(_) => {
                error!("received unknown message");
                panic!("expected game state to be given from server on init")
            }
        }
    }
}
