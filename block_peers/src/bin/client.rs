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

// Std
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use async_std::task;

// Internal
use block_peers::logging;
use block_peers::net::{ClientMessage, ServerMessage, Socket};
use block_peers::render::{Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use block_peers::scene::{AppLifecycleEvent, GameSoundEvent, Scene};
use block_peers::scenes::TitleScene;
use block_peers::sound::{AudioManager, SoundEffect};

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

    // Draw
    let mut window_builder = video_subsystem.window("Block Wars", WINDOW_WIDTH, WINDOW_HEIGHT);
    window_builder.opengl();

    if options.fullscreen {
        window_builder.fullscreen();
    } else {
        window_builder.position_centered().resizable();
    }

    let window = window_builder.build().unwrap();
    let mut renderer = Renderer::new(
        window.into_canvas().present_vsync().build().unwrap(),
        Path::new("assets/textures.png"),
        Path::new("assets/VT323-Regular.ttf"),
        &ttf,
    );

    // Input
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Timing
    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();
    let mut fps = 0;
    let mut ups = 0;
    let mut fps_timer = Instant::now();

    // Network
    let mut socket =
        task::block_on(async { Socket::new().await }).expect("could not open a new socket");

    // Scene
    let mut scene: Box<dyn Scene> = Box::new(TitleScene::new(server_addr));

    // Audio
    let mut audio_manager = AudioManager::new();
    let mut sound_events = Vec::new();
    if options.no_sound {
        audio_manager.dev_turn_sound_off();
    }
    audio_manager.set_volume(0.20);
    audio_manager.play_bg_music();
    let (tx, rx) = channel::<(SocketAddr, ServerMessage)>();
    let (mut sx, sr) = channel::<(SocketAddr, ClientMessage)>();

    task::spawn(async move {
        loop {
            if let Ok(Some((addr, content))) = socket.receive::<ServerMessage>().await {
                tx.send((addr, content));
            }

            if let Ok((addr, message)) = sr.try_recv() {
                println!("sending via socket");
                socket.send(addr, message);
            }
        }
    });

    'running: loop {
        // Network
        loop {
            match rx.try_recv() {
                Ok((source_addr, message)) => {
                    println!("received message, {:?}", source_addr);
                    scene = scene.handle_message(&mut sx, source_addr, message);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(e) => {
                    error!("received unknown message: {:?}", e);
                    break 'running;
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
                    scene.lifecycle(&mut sx, AppLifecycleEvent::Shutdown);
                    break 'running;
                }

                event => {
                    scene = scene.input(&mut sx, event);
                }
            }
        }

        // Update
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            scene = scene.update(&mut sx, &mut sound_events);
            previous_instant += tick_duration;
            ups += 1;
        }

        // Handle any sounds due to update
        for event in sound_events.iter() {
            match event {
                GameSoundEvent::LinesCleared(count) => match count {
                    1 => audio_manager.play_sfx(SoundEffect::SmokeOne),
                    2 => audio_manager.play_sfx(SoundEffect::SmokeTwo),
                    3 => audio_manager.play_sfx(SoundEffect::SmokeThree),
                    4 => audio_manager.play_sfx(SoundEffect::SmokeFour),
                    _ => unreachable!("tried to clear illegal number of lines"),
                },
                GameSoundEvent::MovePieceDown => {
                    audio_manager.play_sfx(SoundEffect::Whoosh);
                }
                GameSoundEvent::TurnSoundsOff => {
                    audio_manager.ui_turn_sound_off();
                }
                GameSoundEvent::TurnSoundsOn => {
                    audio_manager.ui_turn_sound_on();
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
    no_sound: bool,
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
    opts.optflag("s", "no-sound", "open the game without sound");

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
    let no_sound: bool = matches.opt_present("no-sound");

    ClientOptions {
        host,
        port,
        fullscreen,
        no_sound,
    }
}
