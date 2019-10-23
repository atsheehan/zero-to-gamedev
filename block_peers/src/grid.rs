use sdl2::pixels::Color;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};

// Internal
use crate::brick::{Brick, BrickIterator, GridCell};
use crate::piece::{random_next_piece, Piece};
use crate::render::{Image, Opacity, Renderer};

const CELL_SIZE: u32 = 20;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Grid {
    height: u32,
    width: u32,
    cells: Vec<Brick>,
    current_piece: Piece,
    drop_counter: u32,
}

impl Grid {
    pub fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let cells = vec![Brick::Empty; cell_count as usize];
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

    pub fn move_piece_left(&mut self) {
        let next = self.current_piece.move_left();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
        }
    }

    pub fn move_piece_right(&mut self) {
        let next = self.current_piece.move_right();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
        }
    }

    pub fn rotate(&mut self) {
        let next = self.current_piece.rotate();
        if self.does_piece_fit(&next) {
            self.current_piece = next;
        }
    }

    pub fn move_piece_down(&mut self) -> bool {
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

    pub fn move_piece_to_bottom(&mut self) {
        while !self.move_piece_down() {}
    }

    fn in_bounds(&self, cell: GridCell) -> bool {
        cell.col >= 0
            && cell.col < self.width as i32
            && cell.row >= 0
            && cell.row < self.height as i32
    }

    fn is_occupied(&self, cell: GridCell) -> bool {
        self.cells[self.cell_index(cell)] != Brick::Empty
    }

    fn cell_index(&self, cell: GridCell) -> usize {
        (cell.row * self.width as i32 + cell.col) as usize
    }

    fn does_piece_fit(&self, piece: &Piece) -> bool {
        piece
            .global_iter()
            .all(|cell| self.in_bounds(cell) && !self.is_occupied(cell))
    }

    fn grid_iterator(&self) -> BrickIterator {
        BrickIterator::new((0, 0), self.width, self.height, self.cells.clone())
    }

    fn attach_piece_to_grid(&mut self) {
        for cell in self.current_piece.global_iter() {
            let idx = self.cell_index(cell);
            self.cells[idx] = Brick::Occupied(self.current_piece.image());
        }
    }

    pub fn update(&mut self) {
        self.drop_counter += 1;

        if self.drop_counter >= 100 {
            self.move_piece_down();
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        // Render board background
        renderer.fill_rect(
            Rect::new(0, 0, self.width * CELL_SIZE, self.height * CELL_SIZE),
            Color::RGB(0, 0, 0),
        );

        // Render occupied cells on the board
        for cell in self.grid_iterator() {
            let idx = self.cell_index(cell);
            match self.cells[idx] {
                Brick::Occupied(image) => {
                    self.render_brick(renderer, cell, image, Opacity::Opaque);
                }
                _ => {}
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
