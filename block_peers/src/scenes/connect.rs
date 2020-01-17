use sdl2::event::Event;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use async_std::task;

use crate::net::{ClientMessage, ServerMessage};
use crate::render::Renderer;
use crate::scene::{GameSoundEvent, Scene};
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
    state: ConnectionState,
    connection_attempt_counter: u64,
}

impl ConnectScene {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
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
    fn input(
        self: Box<Self>,
        _socket: &mut Sender<(SocketAddr, ClientMessage)>,
        _event: Event,
    ) -> Box<dyn Scene> {
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

    fn update(
        mut self: Box<Self>,
        socket: &mut Sender<(SocketAddr, ClientMessage)>,
        _sounds: &mut Vec<GameSoundEvent>,
    ) -> Box<dyn Scene> {
        if self.connection_attempt_counter >= MAX_CONNECTION_ATTEMPTS {
            self.state = ConnectionState::TimedOut;
            return self;
        }

        match self.state {
            ConnectionState::SendingConnectionRequest => {
                debug!("sending connection request");
                socket
                    .send((self.server_addr, ClientMessage::Connect))
                    .unwrap();
                self.connection_attempt_counter += 1;
            }
            ConnectionState::SendingChallengeResponse { salt } => {
                debug!("sending challenge response");
                socket
                    .send((self.server_addr, ClientMessage::ChallengeResponse { salt }))
                    .unwrap();
            }
            _ => {}
        }

        self
    }

    fn handle_message(
        mut self: Box<Self>,
        _socket: &mut Sender<(SocketAddr, ClientMessage)>,
        source_addr: SocketAddr,
        message: ServerMessage,
    ) -> Box<dyn Scene> {
        match message {
            ServerMessage::Sync { player_id, grids } => {
                debug!("connected to server at {:?}", source_addr);

                match self.state {
                    ConnectionState::Connected => Box::new(GameScene::new(
                        player_id,
                        grids.into_owned(),
                        self.server_addr,
                    )),
                    _ => self,
                }
            }
            ServerMessage::ConnectionAccepted => {
                debug!("connection accepted for client {}", source_addr);
                self.state = ConnectionState::Connected;
                self
            }
            ServerMessage::ConnectionRejected => {
                error!("client {} was rejected!", source_addr);
                self.state = ConnectionState::Rejected;
                self
            }
            ServerMessage::Challenge { salt } => {
                debug!("received challenge from server: {}", salt);
                self.state = ConnectionState::SendingChallengeResponse { salt };
                self
            }
        }
    }
}

fn lerp(start: f32, end: f32, time: f32) -> f32 {
    start * (1f32 - time) + end * time
}
