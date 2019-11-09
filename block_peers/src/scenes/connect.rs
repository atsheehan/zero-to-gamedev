use sdl2::event::Event;
use std::net::SocketAddr;

use crate::net::{ClientMessage, ServerMessage, Socket};
use crate::render::Renderer;
use crate::scene::Scene;
use crate::scenes::GameScene;
use crate::text::Text;

// ~10 seconds at 60 fps
const MAX_CONNECTION_ATTEMPTS: u64 = 600;

enum ConnectionState {
    // We're currently trying to connect to he server
    SendingConnectionRequest,
    // We've reached the server but need to finish the secret handshake
    SendingChallengeResponse { salt: u64 },
    // Challenge was successful, time to play game
    Connected,
    // We've been rejected because a game is already going
    Rejected,
    // We've tried connecting to the server for X seconds but have been unable to reach.
    TimedOut,
}

pub struct ConnectScene {
    server_addr: SocketAddr,
    socket: Socket,
    state: ConnectionState,
    connection_attempt_counter: u64,
}

impl ConnectScene {
    pub fn new(server_addr: SocketAddr) -> Self {
        let socket = Socket::new().expect("could not open a new socket");

        Self {
            server_addr,
            socket,
            state: ConnectionState::SendingConnectionRequest,
            connection_attempt_counter: 0,
        }
    }

    fn dots(&self) -> String {
        let num_dots = lerp(
            1f32,
            20f32,
            (self.connection_attempt_counter as f32 / MAX_CONNECTION_ATTEMPTS as f32) as f32,
        ) as u64;

        let mut s = String::new();
        for _ in 0..num_dots {
            s.push_str(".");
        }

        s
    }
}

impl Scene for ConnectScene {
    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        let message: &'static str;

        match self.state {
            ConnectionState::SendingConnectionRequest
            | ConnectionState::SendingChallengeResponse { .. } => {
                message = "Connecting to server";
                renderer.render_text(
                    Text::from(self.dots())
                        .center_xy(400, 340)
                        .height(40)
                        .build(),
                );
            }
            ConnectionState::Connected => {
                message = "Waiting to start game...";
            }
            ConnectionState::TimedOut => {
                message = "Timed Out";
            }
            ConnectionState::Rejected => {
                message = "REJECTED";
            }
        }

        renderer.render_text(Text::new(message).center_xy(400, 300).height(40).build());
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        if self.connection_attempt_counter >= MAX_CONNECTION_ATTEMPTS {
            self.state = ConnectionState::TimedOut;
            return self;
        }

        match self.state {
            ConnectionState::SendingConnectionRequest => {
                debug!("sending connection request");
                self.socket
                    .send(self.server_addr, &ClientMessage::Connect)
                    .unwrap();
                self.connection_attempt_counter += 1;
            }
            ConnectionState::SendingChallengeResponse { salt } => {
                debug!("sending challenge response");
                self.socket
                    .send(self.server_addr, &ClientMessage::ChallengeResponse { salt })
                    .unwrap();
            }
            _ => {}
        }

        match self.socket.receive::<ServerMessage>() {
            Ok(Some((source_addr, ServerMessage::Sync { grid }))) => {
                debug!("connected to server at {:?}", source_addr);

                match self.state {
                    ConnectionState::Connected => Box::new(GameScene::new(
                        grid.into_owned(),
                        self.socket,
                        self.server_addr,
                    )),
                    _ => self,
                }
            }
            Ok(Some((source_addr, ServerMessage::ConnectionAccepted))) => {
                debug!("connection accepted for client {}", source_addr);
                self.state = ConnectionState::Connected;
                self
            }
            Ok(Some((source_addr, ServerMessage::ConnectionRejected))) => {
                error!("client {} was rejected!", source_addr);
                self.state = ConnectionState::Rejected;
                self
            }
            Ok(Some((_source_addr, ServerMessage::Challenge { salt }))) => {
                debug!("received challenge from server: {}", salt);
                self.state = ConnectionState::SendingChallengeResponse { salt };
                self
            }
            ConnectionState::Rejected => {
                // exit early, maybe retry after some time
            }
        }

        self
    }
}

fn lerp(start: f32, end: f32, time: f32) -> f32 {
    start * (1f32 - time) + end * time
}
