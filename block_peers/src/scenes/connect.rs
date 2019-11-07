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
    state: ConnectionState,
}

enum ConnectionState {
    NotStarted,
    Initiated,
    Connected,
    Rejected,
}

impl ConnectScene {
    pub fn new(server_addr: SocketAddr) -> Self {
        let socket = Socket::new().expect("could not open a new socket");

        Self {
            server_addr,
            socket,
            state: ConnectionState::NotStarted,
        }
    }
}

impl Scene for ConnectScene {
    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        match self.state {
            ConnectionState::Rejected => {
                renderer.render_text(Text::new("REJECTED").center_xy(400, 300).height(40).build());
            },
            ConnectionState::Connected => {
                renderer.render_text(Text::new("Connected, waiting for game to start...").center_xy(400, 300).height(40).build());
            },
            _ => {
                renderer.render_text(
                    Text::new("Connecting to server...")
                        .center_xy(400, 300)
                        .height(40)
                        .build(),
                );
            }
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        match self.state {
            ConnectionState::NotStarted => {
                self.socket
                    .send(self.server_addr, &ClientMessage::Connect)
                    .unwrap();

                self.state = ConnectionState::Initiated;
            }
            ConnectionState::Initiated => {
                // wait for connected message, transition to connected
                match self.socket.receive::<ServerMessage>() {
                    Ok(Some((source_addr, ServerMessage::Connected))) => {
                        debug!("connected to server at {:?}", source_addr);
                        self.state = ConnectionState::Connected;
                    }
                    Ok(Some((source_addr, ServerMessage::Reject))) => {
                        error!("client {} was rejected!", source_addr);
                        self.state = ConnectionState::Rejected;
                    }
                    Ok(None) => {},
                    Ok(Some((_source_addr, message))) => {
                        debug!("received unexpected message: {:?}", message);
                    }
                    Err(_) => {
                        error!("received unknown message");
                        panic!("expected game state to be given from server on init");
                    }
                }
            }
            ConnectionState::Connected => {
                // wait for sync message, transition to new game state
                match self.socket.receive::<ServerMessage>() {
                    Ok(Some((source_addr, ServerMessage::Sync { grids }))) => {
                        debug!("connected to server at {:?}", source_addr);
                        return Box::new(GameScene::new(
                            grids.into_owned(),
                            self.socket,
                            self.server_addr,
                        ));
                    }
                    Ok(Some((source_addr, ServerMessage::Reject))) => {
                        error!("client {} was rejected!", source_addr);
                        self.state = ConnectionState::Rejected;
                    }
                    Ok(None) => {},
                    Ok(Some((_source_addr, message))) => {
                        debug!("received unexpected message: {:?}", message);
                    }
                    Err(_) => {
                        error!("received unknown message");
                        panic!("expected game state to be given from server on init");
                    }
                }
            }
            ConnectionState::Rejected => {
                // exit early, maybe retry after some time
            }
        }

        self
    }
}
