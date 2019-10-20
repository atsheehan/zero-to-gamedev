#[macro_use]
extern crate log;
extern crate rand;
extern crate sdl2;
extern crate simplelog;

mod brick;
mod piece;
mod render;
mod util;

// External
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};

// Internal
use brick::{BrickIterator, GridCell};
use piece::{random_next_piece, Piece};
use render::Renderer;

// Constants
const IS_DEBUG: bool = true;
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
        let cells = vec![false; cell_count as usize];
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
        for cell in piece.global_iter() {
            if cell.in_bounds(self.width as i32, self.height as i32) {
                let grid_index = cell.row * self.width as i32 + cell.col;

                if self.cells[grid_index as usize] {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn grid_iterator(&self) -> BrickIterator {
        BrickIterator::new((0, 0), self.width, self.height, self.cells.clone())
    }

    fn above_line_iterator(&self, line_num: u32) -> BrickIterator {
        BrickIterator::new((0, 0), self.width, line_num, self.cells.clone())
    }

    fn attach_piece_to_grid(&mut self) {
        for GridCell { row, col } in self.current_piece.local_iter() {
            let (x_offset, y_offset) = self.current_piece.origin();
            let x = col + x_offset;
            let y = row + y_offset;
            let grid_index = y * self.width as i32 + x;

            self.cells[grid_index as usize] = true
        }

        self.clear_full_lines();
    }

    // This is a pretttty ugly implementation haha. Not entirely sure on how we want blocks to
    // actually fall. Here is an article I found with different ways of approaching it:
    //
    // https://gamedevelopment.tutsplus.com/tutorials/implementing-tetris-clearing-lines--gamedev-1197
    fn move_bricks_down(&mut self, above_line: i32) {
        let mut more_to_move = true;

        while more_to_move {
            let mut stale_idx: Vec<i32> = vec![];
            let mut new_idx: Vec<i32> = vec![];

            // Move all active bricks above the line down one
            for cell in self.above_line_iterator(above_line as u32) {
                let new_cell = cell + GridCell { col: 0, row: 1 };

                if new_cell.in_bounds(self.width as i32, self.height as i32) {
                    let new_index = new_cell.row * self.width as i32 + cell.col;
                    if !self.cells[new_index as usize] {
                        let original_index = cell.row * self.width as i32 + cell.col;
                        stale_idx.push(original_index);
                        new_idx.push(new_index);
                    }
                }
            }

            for idx in stale_idx {
                self.cells[idx as usize] = false;
            }
            for idx in new_idx {
                self.cells[idx as usize] = true;
            }

            // Detect if any other bricks still need to be moved down or not.
            let mut any_brick_can_go_down = false;
            for cell in self.above_line_iterator(above_line as u32) {
                let new_cell = cell + GridCell { col: 0, row: 1 };

                if new_cell.in_bounds(self.width as i32, self.height as i32) {
                    let new_index = new_cell.row * self.width as i32 + cell.col;

                    if !self.cells[new_index as usize] {
                        any_brick_can_go_down = true;
                    }
                }
            }
            more_to_move = any_brick_can_go_down;
        }

        // TODO: Do we clear lines that were made from falling at the same time??
        self.clear_full_lines();
    }

    fn clear_full_lines(&mut self) {
        // TODO: Maybe create some sort of line iterator that iterates from bottom up?
        let mut row = self.height - 1;

        while row > 0 {
            let mut full_line = true;
            for col in 0..self.width {
                let index = row * self.width + col;
                full_line &= self.cells[index as usize];
            }

            if full_line {
                for col in 0..self.width {
                    let index = row * self.width + col;
                    self.cells[index as usize] = false;
                }
                // Move bricks down that are above this line
                self.move_bricks_down(row as i32);
            }

            row -= 1;
        }
    }

    fn update(&mut self) {
        self.drop_counter += 1;

        if self.drop_counter >= 100 {
            self.move_piece_down();
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        // Render board background
        renderer.fill_rect(
            Rect::new(0, 0, self.width * CELL_SIZE, self.height * CELL_SIZE),
            Color::RGB(0, 0, 0),
        );

        // Render occupied cells on the board
        for cell in self.grid_iterator() {
            self.render_brick(renderer, cell, Color::RGB(255, 255, 255));
        }

        // Render current piece
        let piece_color = Color::RGB(255, 255, 255);
        self.render_piece(renderer, &self.current_piece, piece_color);

        // Render ghost piece
        let ghost_color = Color::RGB(125, 125, 125);
        let mut ghost_piece = self.current_piece.move_down();
        let mut next_ghost_piece = ghost_piece.move_down();

        while self.does_piece_fit(&next_ghost_piece) {
            ghost_piece = next_ghost_piece;
            next_ghost_piece = ghost_piece.move_down();
        }
        self.render_piece(renderer, &ghost_piece, ghost_color);

        // Render full lines for debugging
        if IS_DEBUG {
            for row in 0..self.height {
                let mut full_line = true;

                for col in 0..self.width {
                    let index = row * self.width + col;
                    full_line &= self.cells[index as usize];
                }

                if full_line {
                    for col in 0..self.width {
                        let cell = GridCell {
                            col: col as i32,
                            row: row as i32,
                        };
                        self.render_brick(renderer, cell, Color::RGB(255, 40, 40));
                    }
                }
            }
        }
    }

    fn render_piece(&self, renderer: &mut Renderer, piece: &Piece, color: Color) {
        for GridCell { col, row } in piece.local_iter() {
            let (x_offset, y_offset) = piece.origin();
            let x = (col + x_offset) * CELL_SIZE as i32;
            let y = (row + y_offset) * CELL_SIZE as i32;
            renderer.fill_rect(Rect::new(x, y, CELL_SIZE, CELL_SIZE), color);
        }
    }

    fn render_brick(&self, renderer: &mut Renderer, cell: GridCell, color: Color) {
        let x = cell.col as u32 * CELL_SIZE;
        let y = cell.row as u32 * CELL_SIZE;

        renderer.fill_rect(Rect::new(x as i32, y as i32, CELL_SIZE, CELL_SIZE), color);
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
    let mut renderer = Renderer::new(window.into_canvas().present_vsync().build().unwrap());

    // Input
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Timing
    let tick_duration = Duration::from_micros(MICROSECONDS_PER_TICK);
    let mut previous_instant = Instant::now();
    let mut fps = 0;
    let mut ups = 0;
    let mut fps_timer = Instant::now();

    // Game State
    let mut grid = Grid::new(20, 10);

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
