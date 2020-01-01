use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::net::SocketAddr;
use std::slice::Iter;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ai::DumbAI;
use crate::brick::CELL_SIZE;
use crate::grid::Grid;
use crate::image::Image;
use crate::net::{ServerMessage, Socket};
use crate::piece::Piece;
use bananas::render::{Opacity, Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::scene::{GameSoundEvent, Scene};
use crate::scenes::ConnectScene;
use crate::sound::SOUND_IS_ENABLED;
use bananas::text::Text;

const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct TitleScene {
    server_addr: SocketAddr,
    ai: DumbAI,
    state: MenuState,
    should_quit: bool,
    should_stop_sounds: bool,
    should_start_sounds: bool,
}

impl TitleScene {
    pub fn new(server_addr: SocketAddr) -> Self {
        let width = VIEWPORT_WIDTH / CELL_SIZE;
        let height = VIEWPORT_HEIGHT / CELL_SIZE;
        let mut background_grid = Grid::new(height, width);

        // Set some pieces on the board
        background_grid.place_piece_at_bottom(Piece::new(6).move_left());
        background_grid.place_piece_at_bottom(Piece::new(0).rotate().rotate().move_right_times(2));
        background_grid.place_piece_at_bottom(Piece::new(2));
        background_grid.place_piece_at_bottom(Piece::new(2).move_right_times(4));
        background_grid.place_piece_at_bottom(Piece::new(4).move_right_times(2));
        background_grid.place_piece_at_bottom(Piece::new(1).rotate().move_right_times(8));

        Self {
            server_addr,
            ai: DumbAI::new(background_grid),
            state: MenuState::StartGame,
            should_quit: false,
            should_stop_sounds: false,
            should_start_sounds: false,
        }
    }
}

impl Scene for TitleScene {
    fn input(mut self: Box<Self>, _socket: &mut Socket, event: Event) -> Box<dyn Scene> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => match self.state {
                MenuState::StartGame => Box::new(ConnectScene::new(self.server_addr)),
                MenuState::ToggleSound => {
                    if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
                        SOUND_IS_ENABLED.store(false, Ordering::Relaxed);
                        self.should_stop_sounds = true;
                    } else {
                        SOUND_IS_ENABLED.store(true, Ordering::Relaxed);
                        self.should_start_sounds = true;
                    }

                    self
                }
                MenuState::Quit => {
                    self.should_quit = true;
                    self
                }
            },
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                self.state = self.state.previous();
                self
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                self.state = self.state.next();
                self
            }
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(width, height) => {
                    info!("New window dimensions: {} {}", width, height);
                    warn!("TODO: fix the grid and dimensions of menu items being off");
                    self
                }
                _ => self,
            },
            _ => self,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        // Render AI Background
        self.ai.render(renderer);

        // Render title
        renderer.render_image(Image::Title, Rect::new(160, 120, 480, 64), Opacity::Opaque);

        // Render Menu
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let in_ms = since_the_epoch.as_millis() as f64;
        let mut t = (in_ms * 3.0).cos();
        t *= t;
        t = 0.4 + 0.54 * t;
        let white = (255.0, 255.0, 255.0);
        let red = (234.0, 77.0, 72.0);
        let color = lerp(red, white, t);

        let selected_bg_color = Color::RGB(color.0 as u8, color.1 as u8, color.2 as u8);
        let selected_color = Color::RGB(234, 77, 72);
        let non_selected_color = Color::RGB(255, 255, 255);
        let non_selected_bg_color = Color::RGB(10, 10, 10);

        for (idx, item) in MenuState::iter().enumerate() {
            let y = 300 + (idx * 50);
            let color = if *item == self.state {
                selected_color
            } else {
                non_selected_color
            };
            let bg_color = if *item == self.state {
                selected_bg_color
            } else {
                non_selected_bg_color
            };

            // Drop shadow
            renderer.with_relative_offset(-1, 2, |renderer| {
                renderer.render_text(
                    Text::new(item.text())
                        .center_xy(400, y as i32)
                        .height(40)
                        .color(bg_color)
                        .build(),
                )
            });

            // Primary text
            renderer.render_text(
                Text::new(item.text())
                    .center_xy(400, y as i32)
                    .height(40)
                    .color(color)
                    .build(),
            );
        }

        // Render Sound Setting
        let sound_text = if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
            "Sound: ON"
        } else {
            "Sound: OFF"
        };
        renderer.render_text(
            Text::new(sound_text)
                .left_top_xy(10, 5)
                .height(30)
                .color(Color::RGB(128, 128, 128))
                .build(),
        );

        let mut version = String::from("Version: ");
        version.push_str(GAME_VERSION);

        renderer.render_text(
            Text::from(version)
                .left_top_xy((VIEWPORT_WIDTH - 180) as i32, 5)
                .height(30)
                .color(Color::RGB(128, 128, 128))
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
        mut self: Box<Self>,
        _socket: &mut Socket,
        sounds: &mut Vec<GameSoundEvent>,
    ) -> Box<dyn Scene> {
        self.ai.update();

        if self.should_stop_sounds {
            sounds.push(GameSoundEvent::TurnSoundsOff);
            self.should_stop_sounds = false;
        } else if self.should_start_sounds {
            sounds.push(GameSoundEvent::TurnSoundsOn);
            self.should_start_sounds = false;
        }

        self
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

#[derive(Debug, PartialEq)]
enum MenuState {
    StartGame,
    ToggleSound,
    Quit,
}

impl MenuState {
    fn iter() -> Iter<'static, MenuState> {
        use MenuState::*;
        static STATES: [MenuState; 3] = [StartGame, ToggleSound, Quit];
        STATES.iter()
    }

    fn text(&self) -> &'static str {
        use MenuState::*;
        match self {
            StartGame => "Start Game",
            ToggleSound => "Toggle Sound",
            Quit => "Quit",
        }
    }

    fn next(&self) -> Self {
        use MenuState::*;
        match self {
            StartGame => ToggleSound,
            ToggleSound => Quit,
            Quit => StartGame,
        }
    }

    fn previous(&self) -> Self {
        use MenuState::*;
        match self {
            StartGame => Quit,
            ToggleSound => StartGame,
            Quit => ToggleSound,
        }
    }
}

fn lerp(start: (f64, f64, f64), end: (f64, f64, f64), time: f64) -> (f64, f64, f64) {
    (
        start.0 * (1f64 - time) + end.0 * time,
        start.1 * (1f64 - time) + end.1 * time,
        start.2 * (1f64 - time) + end.2 * time,
    )
}
