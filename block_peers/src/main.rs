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
use brick::{BrickIterator, GridCell};
use piece::{random_next_piece, Piece};

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
    drop_counter: u32,
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

        // Move piece to right a bit to center it
        let current_piece = random_next_piece().move_right().move_right();

        Self {
            height,
            width,
            cells,
            current_piece,
            drop_counter: 0,
        }
    }

    fn move_piece_left(&mut self) {
        let next = self.current_piece.move_left();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
        }
    }

    fn move_piece_right(&mut self) {
        let next = self.current_piece.move_right();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
        }
    }

    fn rotate(&mut self) {
        let next = self.current_piece.rotate();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
        }
    }

    fn move_piece_down(&mut self) -> bool {
        self.drop_counter = 0;

        let next = self.current_piece.move_down();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
            false
        } else {
            self.attach_piece_to_grid();
            self.current_piece = random_next_piece().move_right().move_right();
            true
        }
    }

    fn move_piece_to_bottom(&mut self) {
        while !self.move_piece_down() {}
    }

    fn does_piece_fit(&self, piece: &Piece) -> bool {
        for GridCell { row, col } in piece.local_iter() {
            let (x_offset, y_offset) = piece.origin();
            let x = col + x_offset;
            let y = row + y_offset;
            let grid_index = y * self.width as i32 + x;

            if self.cells[grid_index as usize] {
                return false;
            }
        }

        true
    }

    fn grid_iterator(&self, only_occupied: bool) -> BrickIterator {
        BrickIterator::new(
            (0, 0),
            self.width,
            self.height,
            self.cells.clone(),
            only_occupied,
        )
    }

    fn attach_piece_to_grid(&mut self) {
        for GridCell { row, col } in self.current_piece.local_iter() {
            let (x_offset, y_offset) = self.current_piece.origin();
            let x = col + x_offset;
            let y = row + y_offset;
            let grid_index = y * self.width as i32 + x;

            self.cells[grid_index as usize] = true
        }
    }

    fn update(&mut self) {
        self.drop_counter += 1;

        if self.drop_counter >= 100 {
            self.move_piece_down();
        }
    }

    fn render(&self, canvas: &mut WindowCanvas) {
        // Render board background
        for cell in self.grid_iterator(false) {
            self.render_brick(canvas, cell, Color::RGB(0, 0, 0));
        }

        // Render occupied cells on the board
        for cell in self.grid_iterator(true) {
            self.render_brick(canvas, cell, Color::RGB(255, 255, 255));
        }

        // Render current piece
        let piece_color = Color::RGB(255, 255, 255);
        self.render_piece(canvas, &self.current_piece, piece_color);

        // Render ghost piece
        let ghost_color = Color::RGB(125, 125, 125);
        let mut ghost_piece = self.current_piece.move_down();
        while self.does_piece_fit(&ghost_piece) {
            ghost_piece = ghost_piece.move_down();
        }
        ghost_piece = ghost_piece.move_up();
        self.render_piece(canvas, &ghost_piece, ghost_color);
    }

    fn render_piece(&self, canvas: &mut WindowCanvas, piece: &Piece, color: Color) {
        for GridCell { col, row } in piece.local_iter() {
            let (x_offset, y_offset) = piece.origin();
            let x = (col + x_offset) * CELL_SIZE as i32;
            let y = (row + y_offset) * CELL_SIZE as i32;
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(x, y, CELL_SIZE, CELL_SIZE))
                .expect("failed to render piece");
        }
    }

    fn render_brick(&self, canvas: &mut WindowCanvas, cell: GridCell, color: Color) {
        let x = cell.col as u32 * CELL_SIZE;
        let y = cell.row as u32 * CELL_SIZE;

        canvas.set_draw_color(color);
        canvas
            .fill_rect(Rect::new(x as i32, y as i32, CELL_SIZE, CELL_SIZE))
            .expect("failed to render brick");
    }
}

pub fn main() {
    // Subsystems Init
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

    // Input
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
