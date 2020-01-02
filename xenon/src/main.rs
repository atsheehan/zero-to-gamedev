use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect as SDLRect;
use sdl2::keyboard::Keycode;
use std::path::Path;
use std::time::{Duration, Instant};
use bananas::render::{Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

const WINDOW_WIDTH: u32 = VIEWPORT_WIDTH;
const WINDOW_HEIGHT: u32 = VIEWPORT_HEIGHT;
const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

enum AppLifecycleEvent {
    Shutdown,
}

trait Scene {
    fn lifecycle(self: Box<Self>, event: AppLifecycleEvent) -> Box<dyn Scene>;
    fn input(self: Box<Self>, event: Event) -> Box<dyn Scene>;
    fn render(&self, renderer: &mut Renderer);
    fn update(self: Box<Self>) -> Box<dyn Scene>;
    fn should_quit(&self) -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug)]
struct Vec2D {
    x: f32,
    y: f32,
}

impl Vec2D {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Debug)]
struct Rect {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Rect {
    fn from_components(position: Vec2D, dimensions: Vec2D) -> Self {
        Self {
            left: position.x,
            top: position.y,
            width: dimensions.x,
            height: dimensions.y,
        }
    }
}

impl Into<SDLRect> for Rect {
    fn into(self) -> SDLRect {
        SDLRect::new(self.left as i32, self.top as i32, self.width as u32, self.height as u32)
    }
}

struct Entity {
    position: Vec2D,
    dimensions: Vec2D,
}

impl Entity {
    fn new(position: Vec2D, dimensions: Vec2D) -> Self {
        Self {
            position,
            dimensions,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::from_components(self.position, self.dimensions)
    }
}

struct WorldScene {
    entities: Vec<Entity>,
}

impl WorldScene {
    fn new() -> Self {
        let entities = vec![
            Entity::new(Vec2D::new(0.0, 0.0), Vec2D::new(10.0, 10.0)),
            Entity::new(Vec2D::new(10.0, 0.0), Vec2D::new(10.0, 10.0)),
            Entity::new(Vec2D::new(20.0, 0.0), Vec2D::new(10.0, 10.0)),
        ];

        Self {
            entities,
        }
    }
}

impl Scene for WorldScene {
    fn lifecycle(self: Box<Self>, _event: AppLifecycleEvent) -> Box<dyn Scene> {
        self
    }

    fn input(self: Box<Self>, _event: Event) -> Box<dyn Scene> {
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        let color = Color::RGB(255, 255, 255);
        for entity in self.entities.iter() {
            renderer.draw_rect(entity.bounds().into(), color);
        }
    }

    fn update(self: Box<Self>) -> Box<dyn Scene> {
        self
    }
}

pub fn main() {
    // Subsystems Init
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();
    let ttf = sdl2::ttf::init().unwrap();

    // Draw
    let window = video_subsystem.window("Xenon", WINDOW_WIDTH, WINDOW_HEIGHT)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

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

    // Scene
    let mut scene: Box<dyn Scene> = Box::new(WorldScene::new());

    'running: loop {
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
                    scene.lifecycle(AppLifecycleEvent::Shutdown);
                    break 'running;
                }

                event => {
                    scene = scene.input(event);
                }
            }
        }

        // Update
        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            scene = scene.update();
            previous_instant += tick_duration;
        }

        if scene.should_quit() {
            break 'running;
        }

        // Render
        renderer.clear();
        scene.render(&mut renderer);
        renderer.present();
    }
}
