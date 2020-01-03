mod math;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;
use std::path::Path;
use std::time::{Duration, Instant};
use bananas::render::{Renderer, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use math::{Vec2D, Rect};

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
    target: Vec2D,
    entities: Vec<Entity>,
}

impl WorldScene {
    fn new() -> Self {
        let entities = vec![
            Entity::new(Vec2D::new(0.0, 0.0), Vec2D::new(10.0, 10.0)),
            Entity::new(Vec2D::new(10.0, 0.0), Vec2D::new(10.0, 10.0)),
            Entity::new(Vec2D::new(20.0, 0.0), Vec2D::new(10.0, 10.0)),
        ];

        let target = Vec2D::new(200.0, 200.0);

        Self {
            entities,
            target,
        }
    }
}

impl Scene for WorldScene {
    fn lifecycle(self: Box<Self>, _event: AppLifecycleEvent) -> Box<dyn Scene> {
        self
    }

    fn input(mut self: Box<Self>, event: Event) -> Box<dyn Scene> {
        match event {
            Event::MouseButtonDown { x, y, .. } => {
                self.target = Vec2D::new(x as f32, y as f32);
            },
            _ => {},
        }


        self
    }

    fn render(&self, renderer: &mut Renderer) {
        let color = Color::RGB(255, 255, 255);
        for entity in self.entities.iter() {
            renderer.draw_rect(entity.bounds().into(), color);
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Scene> {
        let num_entities = self.entities.len();

        for i in 0..num_entities {
            let new_position = {
                let entity = &self.entities[i];

                let vel = (self.target - entity.bounds().center()).normalize();
                let mut new_bounds = entity.bounds().translate(vel);

                let mut j = 0;
                while j < num_entities {
                    if i != j {
                        let other_entity = &self.entities[j];

                        if other_entity.bounds().overlaps(new_bounds) {
                            new_bounds = entity.bounds();
                            break;
                        }
                    }

                    j += 1;
                }

                new_bounds.position()
            };

            self.entities[i].position = new_position;
        }

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
