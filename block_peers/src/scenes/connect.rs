use sdl2::event::Event;
use std::net::SocketAddr;

use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::Scene;
use crate::scenes::GameScene;

pub struct ConnectScene {
    server_addr: SocketAddr,
    socket: Socket,
}

impl ConnectScene {
    pub fn new(server_addr: SocketAddr) -> Self {
        let socket = Socket::new().expect("could not open a new socket");

        Self {
            server_addr,
            socket,
        }
    }
}

impl Scene for ConnectScene {
    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, _renderer: &mut Renderer) {}

    fn update(&mut self) -> Option<Box<dyn Scene>> {
        self.socket
            .send(self.server_addr, &ClientMessage::Connect)
            .unwrap();

        let grid = match self.socket.receive::<ServerMessage>() {
            Ok((source_addr, ServerMessage::Ack { grid })) => {
                debug!("connected to server at {:?}", source_addr);
                grid
            }
            Err(_) => {
                error!("received unknown message");
                panic!("expected game state to be given from server on init")
            }
        };

        Some(Box::new(GameScene::new(grid)))
    }
}
