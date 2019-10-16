#[macro_use]
extern crate log;
extern crate sdl2;
extern crate simplelog;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::{Duration, Instant};

mod util;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const CELL_SIZE: u32 = 20;

const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

struct BrickIterator {
    origin: (i32, i32),
    num_columns: u32,
    num_rows: u32,
    current_col: i32,
    current_row: i32,
    cells: Vec<bool>,
}

impl BrickIterator {
    fn new(origin: (i32, i32), num_columns: u32, num_rows: u32, cells: Vec<bool>) -> Self {
        BrickIterator {
            origin,
            num_columns,
            num_rows,
            current_col: 0,
            current_row: 0,
            cells,
        }
    }
}

impl Iterator for BrickIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_row < self.num_rows as i32 {
            while self.current_col < self.num_columns as i32 {
                let index =
                    ((self.current_row * self.num_columns as i32) + self.current_col) as usize;

                if self.cells[index] {
                    let (col_offset, row_offset) = self.origin;
                    let col = self.current_col + col_offset;
                    let row = self.current_row + row_offset;

                    self.current_col += 1;
                    return Some((col, row));
                } else {
                    self.current_col += 1;
                }
            }

            self.current_row += 1;
            self.current_col = 0;
        }

        None
    }
}

struct Grid {
    height: u32,
    width: u32,
    cells: Vec<bool>,
    current_piece: Vec<bool>,
    current_piece_origin: (i32, i32),
    drop_counter: u32,
}

impl Grid {
    fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let cells = vec![false; cell_count as usize];

        let mut current_piece = vec![false; 16];
        let current_piece_origin = (2, 0);

        current_piece[1] = true;
        current_piece[2] = true;
        current_piece[3] = true;
        current_piece[6] = true;

        Self {
            height,
            width,
            cells,
            current_piece,
            current_piece_origin,
            drop_counter: 0,
        }
    }

    fn move_piece_left(&mut self) {
        let (x_offset, _) = self.current_piece_origin;
        let x_offset = x_offset - 1;

        for col in 0..4 {
            for row in 0..4 {
                let index = (row * 4) + col;

                // brick is occupied
                if self.current_piece[index] {
                    let x = col as i32 + x_offset;
                    if x < 0 || x >= self.width as i32 {
                        return;
                    }
                }
            }
        }

        self.current_piece_origin.0 -= 1;
    }

    fn move_piece_right(&mut self) {
        let (x_offset, _) = self.current_piece_origin;
        let x_offset = x_offset + 1;

        for col in 0..4 {
            for row in 0..4 {
                let index = (row * 4) + col;

                // brick is occupied
                if self.current_piece[index] {
                    let x = col as i32 + x_offset;
                    if x < 0 || x >= self.width as i32 {
                        return;
                    }
                }
            }
        }

        self.current_piece_origin.0 += 1;
    }

    fn move_piece_down(&mut self) -> bool {
        self.drop_counter = 0;

        let (x_offset, y_offset) = self.current_piece_origin;
        let y_offset = y_offset + 1;

        let next_piece_origin = (x_offset, y_offset);

        if self.is_colliding(next_piece_origin) {
            self.attach_piece_to_grid();
            self.current_piece_origin = (2, 0);
            true
        } else {
            self.current_piece_origin = next_piece_origin;
            false
        }
    }

    fn move_piece_to_bottom(&mut self) {
        while !self.move_piece_down() {}
    }

    fn piece_iterator(&self, origin: (i32, i32)) -> BrickIterator {
        BrickIterator::new(origin, 4, 4, self.current_piece.clone())
    }

    fn is_colliding(&self, piece_origin: (i32, i32)) -> bool {
        for (col, row) in self.piece_iterator(piece_origin) {
            let grid_index = ((row * self.width as i32) + col) as usize;

            if row >= self.height as i32 || self.cells[grid_index] {
                return true;
            }
        }

        false
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
                let index = (row * 4) + col;

                if self.current_piece[index as usize] {
                    let color = Color::RGB(255, 255, 255);

                    // determine cell size
                    let (x_offset, y_offset) = self.current_piece_origin;
                    let x = (col + x_offset) * CELL_SIZE as i32;
                    let y = (row + y_offset) * CELL_SIZE as i32;

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
    util::init_logging();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Block Peers", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();

    // Grids
    let mut grid = Grid::new(20, 10);

    // Debug
    let mut fps = 0;
    let mut ups = 0;
    let mut fps_timer = Instant::now();

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
