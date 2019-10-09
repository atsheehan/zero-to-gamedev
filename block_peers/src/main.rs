extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::{Duration, Instant};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const CELL_SIZE: u32 = 20;

const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

struct Grid {
    height: u32,
    width: u32,
    cells: Vec<bool>,
}

impl Grid {
    fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let mut cells = vec![false; cell_count as usize];

        cells[3] = true;
        cells[4] = true;
        cells[5] = true;
        cells[14] = true;

        Self { height, width, cells }
    }

    fn render(&self, canvas: &mut WindowCanvas) {
        for col in 0..self.width {
            for row in 0..self.height {
                let index = (row * self.width) + col;
                let color = if self.cells[index as usize] {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(0, 0, 0)
                };

                // determine cell size
                let x = col * CELL_SIZE;
                let y = row * CELL_SIZE;

                canvas.set_draw_color(color);
                canvas
                    .fill_rect(Rect::new(x as i32, y as i32, CELL_SIZE, CELL_SIZE))
                    .expect("failed rect draw");
            }
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Block Peers", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    // Grids
    let grid = Grid::new(20, 10);

    'running: loop {
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
                // Handle other input here
                _ => {}
            }
        }

        let current_instant = Instant::now();
        while current_instant - previous_instant >= tick_duration {
            // Update world here
            previous_instant += tick_duration;
        }

        canvas.set_draw_color(Color::RGB(75, 75, 75));
        canvas.clear();

        // Render world here
        grid.render(&mut canvas);

        canvas.present();
    }
}
