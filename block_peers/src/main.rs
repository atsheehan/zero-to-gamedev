#[macro_use]
extern crate log;
extern crate rand;
extern crate sdl2;
extern crate simplelog;

mod brick;
mod piece;
mod util;

// External
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::{Duration, Instant};

// Internal
use brick::BrickIterator;
use piece::{random_next_piece, rotated_index, Piece, Rotation};

// Constants
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
    current_piece: Piece,
    current_piece_origin: (i32, i32),
    drop_counter: u32,
    rotation: Rotation,
}

impl Grid {
    fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let mut cells = vec![false; cell_count as usize];

        // Set border of our board to white
        for x in 0..width {
            for y in 0..height {
                let index = (y * width + x) as usize;
                cells[index] = x == 0 || x == width - 1 || y == height - 1;
            }
        }
        let current_piece_origin = (2, 0);

        Self {
            height,
            width,
            cells,
            current_piece: random_next_piece(),
            current_piece_origin,
            drop_counter: 0,
            rotation: Rotation::Zero,
        }
    }

    fn move_piece_left(&mut self) {
        let mut next = self.current_piece_origin.clone();
        next.0 -= 1;

        if self.does_piece_fit(self.current_piece.to_vec(), self.rotation.clone(), next) {
            self.current_piece_origin.0 -= 1;
        }
    }

    fn move_piece_right(&mut self) {
        let mut next = self.current_piece_origin.clone();
        next.0 += 1;

        if self.does_piece_fit(self.current_piece.to_vec(), self.rotation.clone(), next) {
            self.current_piece_origin.0 += 1;
        }
    }

    fn move_piece_down(&mut self) -> bool {
        self.drop_counter = 0;

        let mut next = self.current_piece_origin.clone();
        next.1 += 1;

        if self.does_piece_fit(self.current_piece.to_vec(), self.rotation.clone(), next) {
            self.current_piece_origin.1 += 1;
            false
        } else {
            self.attach_piece_to_grid();
            self.current_piece = random_next_piece();
            self.current_piece_origin = (2, 0);
            true
        }
    }

    fn move_piece_to_bottom(&mut self) {
        while !self.move_piece_down() {}
    }

    fn does_piece_fit(&self, piece: Vec<bool>, rotation: Rotation, origin: (i32, i32)) -> bool {
        for x in 0..4 {
            for y in 0..4 {
                let index = rotated_index(x, y, rotation.clone());
                let grid_index = (origin.1 + y as i32) * self.width as i32 + (origin.0 + x as i32);

                // Bounds check
                if origin.0 + x as i32 >= 0 && (origin.0 + x as i32) < (self.width as i32) {
                    if origin.1 + y as i32 >= 0 && (origin.1 + y as i32) < (self.height as i32) {
                        // Collision check
                        if piece[index] && self.cells[grid_index as usize] {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn piece_iterator(&self, origin: (i32, i32)) -> BrickIterator {
        BrickIterator::new(origin, 4, 4, self.current_piece.to_vec().clone())
    }

    fn attach_piece_to_grid(&mut self) {
        for (col, row) in self.piece_iterator(self.current_piece_origin) {
            let index = ((row * self.width as i32) + col) as usize;
            self.cells[index] = true;
        }
    }

    fn render(&self, canvas: &mut WindowCanvas) {
        // Render board
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

        // Render current piece
        for col in 0..4 {
            for row in 0..4 {
                let index = rotated_index(col, row, self.rotation.clone());

                if self.current_piece[index as usize] {
                    let color = Color::RGB(255, 255, 255);

                    // determine cell size
                    let (x_offset, y_offset) = self.current_piece_origin;
                    let x = (col as i32 + x_offset) * CELL_SIZE as i32;
                    let y = (row as i32 + y_offset) * CELL_SIZE as i32;

                    canvas.set_draw_color(color);
                    canvas
                        .fill_rect(Rect::new(x as i32, y as i32, CELL_SIZE, CELL_SIZE))
                        .expect("failed rect draw");
                }
            }
        }
    }

    fn update(&mut self) {
        self.drop_counter += 1;

        if self.drop_counter >= 100 {
            self.move_piece_down();
        }
    }
}

pub fn main() {
    // Subsystems
    util::init_logging();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Draw
    let window = video_subsystem
        .window("Block Peers", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Timing
    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();
    let mut fps = 0;
    let mut ups = 0;
    let mut fps_timer = Instant::now();

    // Game State
    let mut grid = Grid::new(22, 11);

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
                    if grid.does_piece_fit(
                        grid.current_piece.to_vec(),
                        grid.rotation.next(),
                        grid.current_piece_origin,
                    ) {
                        grid.rotation = grid.rotation.next();
                    }
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

        canvas.set_draw_color(Color::RGB(75, 75, 75));
        canvas.clear();

        // Render world here
        grid.render(&mut canvas);
        fps += 1;

        if fps_timer.elapsed().as_millis() >= 1000 {
            debug!("fps {} ups {}", fps, ups);
            fps = 0;
            ups = 0;
            fps_timer = Instant::now();
        }

        canvas.present();
    }
}
