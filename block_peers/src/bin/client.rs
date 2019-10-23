extern crate getopts;
#[macro_use]
extern crate log;
extern crate rand;
extern crate sdl2;
extern crate simplelog;

// External
use getopts::Options;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

// Internal
use block_peers::net::{ClientMessage, ServerMessage};
use block_peers::render::Renderer;
use block_peers::util;

// Constants
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

const DEFAULT_PORT: u16 = 4485;
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub fn main() {
    util::init_logging();

    // TODO: Abstract a bit/clean up
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("p", "port", "connect to server on specified port (default 4485)", "PORT");
    opts.optopt("h", "host", "connect to host at specified address (default 127.0.0.1)", "HOST");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!(f.to_string()) }
    };

    let port: u16 = match matches.opt_get("port") {
        Ok(Some(port)) => { port }
        Ok(None) => { DEFAULT_PORT }
        Err(_) => { panic!("specified port not valid") }
    };

    let host: IpAddr = match matches.opt_get("host") {
        Ok(Some(host)) => { host }
        Ok(None) => { DEFAULT_HOST }
        Err(_) => { panic!("specific host was not valid socket address") }
    };

    let socket = UdpSocket::bind("0.0.0.0:0").expect("could not create a socket");
    let server_addr = SocketAddr::new(host, port);

    let message = ClientMessage::Connect;

    socket.send_to(&message.into_bytes(), &server_addr).unwrap();

    let mut buffer = [0; 1000];
    let (amount, source_addr) = socket.recv_from(&mut buffer).unwrap();

    let data = &buffer[..amount];

    let mut grid = match bincode::deserialize(&data) {
        Ok(ServerMessage::Ack { grid }) => {
            debug!("connected to server at {:?}", source_addr);
            grid
        }
        Err(_) => {
            error!("received unknown message");
            panic!("expected game state to be given from server on init")
        }
    };

    // Subsystems Init
    // Note: handles must stay in scope until end of program due to dropping.
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    // Draw
    let window = video_subsystem
        .window("Block Peers", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = Renderer::new(window.into_canvas().present_vsync().build().unwrap());

    // Input
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Timing
    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();
    let mut fps = 0;
    let mut ups = 0;
    let mut fps_timer = Instant::now();

    'running: loop {
        // Check network for events

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
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    grid.move_piece_left();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    grid.move_piece_right();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    grid.move_piece_down();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    grid.move_piece_to_bottom();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => {
                    grid.rotate();
                }
                _ => {}
            }
        }

        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            grid.update();
            previous_instant += tick_duration;
            ups += 1;
        }

        renderer.clear();

        // Render world here
        grid.render(&mut renderer);
        fps += 1;

        if fps_timer.elapsed().as_millis() >= 1000 {
            debug!("fps {} ups {}", fps, ups);
            fps = 0;
            ups = 0;
            fps_timer = Instant::now();
        }

        renderer.present();
    }
}
