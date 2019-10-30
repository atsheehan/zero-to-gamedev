use sdl2::pixels::Color;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};

// Internal
use crate::brick::{Brick, BrickIterator, BrickType, GridCell, LineIterator, MatchingLine};
use crate::piece::{random_next_piece, Piece};
use crate::render::{Image, Opacity, Renderer};

const CELL_SIZE: u32 = 20;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum GridInputEvent {
    MoveLeft,
    MoveRight,
    MoveDown,
    ForceToBottom,
    Rotate,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Grid {
    height: u32,
    width: u32,
    cells: Vec<Brick>,
    current_piece: Piece,
    drop_counter: u32,
    pub gameover: bool,
}

// ------------
// Public Grid
// ------------
impl Grid {
    pub fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let cells = vec![Brick::Empty; cell_count as usize];
        let current_piece = random_next_piece().center(width);

        Self {
            height,
            width,
            cells,
            current_piece,
            drop_counter: 0,
            gameover: false,
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
            self.spawn_next_piece();
            true
        }
    }

    pub fn move_piece_to_bottom(&mut self) {
        while !self.move_piece_down() {}
    }

    pub fn update(&mut self) {
        // Handle continuous dropping
        self.drop_counter += 1;
        if self.drop_counter >= 100 {
            self.move_piece_down();
        }

        // Increment any outstanding animations
        for cell in self.grid_iterator() {
            let idx = self.cell_index(cell);
            if let Some(next) = self.cells[idx].break_brick() {
                self.cells[idx] = next;
            }
        }

        // Clear finished animations
        for MatchingLine { row, .. } in self.lines_matching(|_, brick| brick.is_broken()) {
            self.move_bricks_down(row as i32);
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
                Brick::Occupied(brick_type) => {
                    let image = Image::from_brick_type(brick_type);
                    self.render_brick(renderer, cell, image, Opacity::Opaque);
                }
                Brick::Breaking(frame) => {
                    let image = Image::from_brick_type(BrickType::Smoke(frame));
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
}

// ------------
// Title Screen
// ------------
//
// These methods are only for convenience in creating the menu title screen animation.
// Avoid using them in the 'real' game.
impl Grid {
    pub fn place_piece_at_bottom(&mut self, piece: Piece) {
        let mut piece = piece;
        let mut next = piece.move_down();

        while self.does_piece_fit(&next) {
            piece = next;
            next = piece.move_down();
        }

        for cell in piece.global_iter() {
            let idx = self.cell_index(cell);
            self.cells[idx] = Brick::Occupied(piece.brick_type());
        }
    }
}

// ------------
// Private Grid
// ------------
impl Grid {
    fn spawn_next_piece(&mut self) {
        let next_piece = random_next_piece().center(self.width);
        if self.does_piece_fit(&next_piece) {
            self.current_piece = random_next_piece().center(self.width);
        } else {
            self.gameover = true;
        }
    }

    fn in_bounds(&self, cell: GridCell) -> bool {
        cell.col >= 0
            && cell.col < self.width as i32
            && cell.row >= 0
            && cell.row < self.height as i32
    }

    fn is_occupied(&self, cell: GridCell) -> bool {
        !self.cells[self.cell_index(cell)].is_empty()
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

    fn lines_matching<CB>(&self, callback: CB) -> LineIterator<CB>
    where
        CB: Fn(GridCell, Brick) -> bool,
    {
        LineIterator::new(self.cells.clone(), self.width, self.height, callback)
    }

    fn attach_piece_to_grid(&mut self) {
        for cell in self.current_piece.global_iter() {
            let idx = self.cell_index(cell);
            self.cells[idx] = Brick::Occupied(self.current_piece.brick_type());
        }
        self.animate_full_lines();
    }

    fn animate_full_lines(&mut self) {
        for MatchingLine { cells, .. } in self.lines_matching(|_, brick| !brick.is_empty()) {
            for cell in cells {
                let idx = self.cell_index(cell);
                self.cells[idx] = Brick::Breaking(0);
            }
        }
    }

    fn move_bricks_down(&mut self, line: i32) {
        for row in (0..line).into_iter().rev() {
            for col in 0..self.width {
                let cell = GridCell {
                    col: col as i32,
                    row: row,
                };
                let new_cell = cell + GridCell { col: 0, row: 1 };

                if self.in_bounds(new_cell) {
                    let old_idx = self.cell_index(cell);
                    let old_content = self.cells[old_idx];
                    let idx = self.cell_index(new_cell);
                    self.cells[idx] = old_content;
                }
            }
        }
        self.clear_full_lines();
    }

    fn clear_full_lines(&mut self) {
        let mut row: i32 = self.height as i32 - 1;

        while row >= 0 {
            let mut full_line = true;
            for col in 0..self.width {
                let cell = GridCell {
                    col: col as i32,
                    row: row,
                };
                full_line &= self.is_occupied(cell);
            }

            if full_line {
                for col in 0..self.width {
                    let cell = GridCell {
                        col: col as i32,
                        row: row,
                    };
                    let idx = self.cell_index(cell);
                    self.cells[idx] = Brick::Empty;
                }
                self.move_bricks_down(row - 1);
            }

            row -= 1;
        }
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
