#[macro_use]
extern crate log;
extern crate getopts;
extern crate rand;
extern crate sdl2;
extern crate simplelog;

// External
use getopts::Options;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self, Channel, Chunk};

// Std
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

// Internal
use block_peers::logging;
use block_peers::net::{ServerMessage, Socket};
use block_peers::render::{Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use block_peers::scene::{AppLifecycleEvent, GameSoundEvent, Scene};
use block_peers::scenes::TitleScene;
use block_peers::sound::SOUND_IS_ENABLED;

// Constants
const WINDOW_WIDTH: u32 = VIEWPORT_WIDTH;
const WINDOW_HEIGHT: u32 = VIEWPORT_HEIGHT;
const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

pub fn main() {
    logging::init();
    let options = get_options();

    let server_addr = SocketAddr::new(options.host, options.port);

    // Subsystems Init
    // Note: handles must stay in scope until end of program due to dropping.
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _audio = sdl_context.audio().unwrap();
    let _image = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();
    let ttf = sdl2::ttf::init().unwrap();

    // Audio
    mixer::open_audio(44_100, mixer::AUDIO_S16LSB, mixer::DEFAULT_CHANNELS, 1_024).unwrap();
    mixer::allocate_channels(4);

    let background_music =
        sdl2::mixer::Music::from_file(Path::new("../../assets/background.wav")).unwrap();
    let smoke_1 = Chunk::from_file(Path::new("../../assets/smoke-1.wav")).unwrap();
    let smoke_2 = Chunk::from_file(Path::new("../../assets/smoke-2.wav")).unwrap();
    let smoke_3 = Chunk::from_file(Path::new("../../assets/smoke-3.wav")).unwrap();
    let smoke_4 = Chunk::from_file(Path::new("../../assets/smoke-4.wav")).unwrap();

    // Volume is between 0-128. 32 will play at quarter volume for chiller background music
    let mut is_paused = false;
    sdl2::mixer::Music::set_volume(32);
    background_music
        .play(std::i32::MAX) // effectively infinitely play in background
        .expect("unable to play background music");

    // Draw
    let mut window_builder = video_subsystem.window("Block Wars", WINDOW_WIDTH, WINDOW_HEIGHT);
    window_builder.opengl();

    if options.fullscreen {
        window_builder.fullscreen();
    } else {
        window_builder.position_centered().resizable();
    }

    let window = window_builder.build().unwrap();
    let mut renderer = Renderer::new(window.into_canvas().present_vsync().build().unwrap(), &ttf);

    // Input
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Timing
    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();
    let mut fps = 0;
    let mut ups = 0;
    let mut fps_timer = Instant::now();

    // Network
    let mut socket = Socket::new().expect("could not open a new socket");

    // Scene
    let mut scene: Box<dyn Scene> = Box::new(TitleScene::new(server_addr));

    // Sound
    let mut sound_events = Vec::new();

    'running: loop {
        // Network
        loop {
            match socket.receive::<ServerMessage>() {
                Ok(Some((source_addr, message))) => {
                    scene = scene.handle_message(&mut socket, source_addr, message);
                }
                Ok(None) => {
                    break;
                }
                Err(_) => {
                    error!("received unknown message");
                }
            }
        }

        // Input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    trace!("app asked to shutdown");
                    scene.lifecycle(&mut socket, AppLifecycleEvent::Shutdown);
                    break 'running;
                }

                event => {
                    scene = scene.input(&mut socket, event);
                }
            }
        }

        // Update
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            scene = scene.update(&mut socket, &mut sound_events);
            previous_instant += tick_duration;
            ups += 1;
        }

        // Handle any sounds due to update
        for event in sound_events.iter() {
            match event {
                GameSoundEvent::LinesCleared(count) => match count {
                    1 => {
                        if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
                            Channel::all().play(&smoke_1, 0).unwrap();
                        }
                    }
                    2 => {
                        if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
                            Channel::all().play(&smoke_2, 0).unwrap();
                        }
                    }
                    3 => {
                        if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
                            Channel::all().play(&smoke_3, 0).unwrap();
                        }
                    }
                    4 => {
                        if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
                            Channel::all().play(&smoke_4, 0).unwrap();
                        }
                    }
                    _ => unreachable!("tried to clear illegal number of lines"),
                },
                GameSoundEvent::TurnSoundsOff => {
                    is_paused = true;
                    sdl2::mixer::Music::halt();
                }
                GameSoundEvent::TurnSoundsOn => {
                    if is_paused {
                        background_music
                            .play(std::i32::MAX) // effectively infinitely play in background
                            .expect("unable to play background music");
                    }
                }
            }
        }

        sound_events.clear();

        if scene.should_quit() {
            break 'running;
        }

        // Render
        renderer.clear();
        scene.render(&mut renderer);

        fps += 1;
        if fps_timer.elapsed().as_millis() >= 1000 {
            trace!("fps {} ups {}", fps, ups);
            fps = 0;
            ups = 0;
            fps_timer = Instant::now();
        }

        renderer.present();
    }
}

const DEFAULT_PORT: u16 = 4485;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

struct ClientOptions {
    port: u16,
    host: IpAddr,
    fullscreen: bool,
}

fn get_options() -> ClientOptions {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt(
        "p",
        "port",
        "connect to server on specified port (default 4485)",
        "PORT",
    );
    opts.optopt(
        "h",
        "host",
        "connect to host at specified address (default 127.0.0.1)",
        "HOST",
    );
    opts.optflag("f", "fullscreen", "open the game in a fullscreen window");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let port: u16 = match matches.opt_get("port") {
        Ok(Some(port)) => port,
        Ok(None) => DEFAULT_PORT,
        Err(_) => panic!("specified port not valid"),
    };

    let host: IpAddr = match matches.opt_get("host") {
        Ok(Some(host)) => host,
        Ok(None) => DEFAULT_HOST,
        Err(_) => panic!("specific host was not valid socket address"),
    };

    let fullscreen: bool = matches.opt_present("fullscreen");

    ClientOptions {
        host,
        port,
        fullscreen,
    }
}
