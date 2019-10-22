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
use render::{Image, Opacity, Renderer};

// Constants
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const CELL_SIZE: u32 = 20;
const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;
const TICKS_PER_SECOND: u64 = 60;
const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const MICROSECONDS_PER_TICK: u64 = MICROSECONDS_PER_SECOND / TICKS_PER_SECOND;

struct Grid {
    height: u32,
    width: u32,
    cells: Vec<bool>,
    cell_images: Vec<Option<Image>>,
    current_piece: Piece,
    drop_counter: u32,
}

impl Grid {
    fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let cells = vec![false; cell_count as usize];
        let cell_images = vec![None; cell_count as usize];
        // Move piece to right a bit to center it
        let current_piece = random_next_piece().move_right().move_right();

        Self {
            height,
            width,
            cells,
            cell_images,
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

    fn in_bounds(&self, cell: GridCell) -> bool {
        cell.col >= 0 && cell.col < self.width as i32 && cell.row >= 0 && cell.row < self.height as i32
    }

    fn is_occupied(&self, cell: GridCell) -> bool {
        self.cells[self.cell_index(cell)]
    }

    fn cell_index(&self, cell: GridCell) -> usize {
        (cell.row * self.width as i32 + cell.col) as usize
    }

    fn does_piece_fit(&self, piece: &Piece) -> bool {
        piece.global_iter().all(|cell| self.in_bounds(cell) && !self.is_occupied(cell))
    }

    fn grid_iterator(&self) -> BrickIterator {
        BrickIterator::new((0, 0), self.width, self.height, self.cells.clone())
    }

    fn attach_piece_to_grid(&mut self) {
        for GridCell { row, col } in self.current_piece.global_iter() {
            let grid_index = row * self.width as i32 + col;

            self.cells[grid_index as usize] = true;
            self.cell_images[grid_index as usize] = Some(self.current_piece.image());
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
            let grid_index = cell.row * self.width as i32 + cell.col;
            if let Some(image) = self.cell_images[grid_index as usize] {
                self.render_brick(renderer, cell, image, Opacity::Opaque);
            }
        }

        // Render current piece
        self.render_piece(renderer, &self.current_piece, Opacity::Opaque);

        // Render ghost piece
        let mut ghost_piece = self.current_piece.move_down();
        let mut next_ghost_piece = ghost_piece.move_down();

        while self.does_piece_fit(&next_ghost_piece) {
            ghost_piece = next_ghost_piece;
            next_ghost_piece = ghost_piece.move_down();
        }

        self.render_piece(renderer, &ghost_piece, Opacity::Translucent(128));
    }

    fn render_piece(&self, renderer: &mut Renderer, piece: &Piece, opacity: Opacity) {
        for GridCell { col, row } in piece.global_iter() {
            let x = col * CELL_SIZE as i32;
            let y = row * CELL_SIZE as i32;
            renderer.render_image(
                piece.image(),
                Rect::new(x, y, CELL_SIZE, CELL_SIZE),
                opacity,
            );
        }
    }

    fn render_brick(
        &self,
        renderer: &mut Renderer,
        cell: GridCell,
        image: Image,
        opacity: Opacity,
    ) {
        let x = cell.col as u32 * CELL_SIZE;
        let y = cell.row as u32 * CELL_SIZE;

        renderer.render_image(
            image,
            Rect::new(x as i32, y as i32, CELL_SIZE, CELL_SIZE),
            opacity,
        );
    }
}

pub fn main() {
    // Subsystems Init
    util::init_logging();
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

    // Game State
    let mut grid = Grid::new(GRID_HEIGHT, GRID_WIDTH);

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
