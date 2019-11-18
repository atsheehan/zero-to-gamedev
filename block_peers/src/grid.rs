use sdl2::pixels::Color;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};

// Internal
use crate::brick::{
    Brick, BrickIterator, BrickType, GridCell, LineIterator, MatchingLine, CELL_SIZE,
};
use crate::piece::{random_next_piece, Piece};
use crate::render::{Image, Opacity, Renderer};
use crate::text::Text;

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
    staged_piece: Piece,
    current_piece: Piece,
    drop_counter: u32,
    pub gameover: bool,
    score: u32,
    hard_drop_count: u32,
}

// ------------
// Public Grid
// ------------
impl Grid {
    pub fn new(height: u32, width: u32) -> Self {
        let cell_count = height * width;
        let cells = vec![Brick::Empty; cell_count as usize];
        let current_piece = random_next_piece().center(width);
        let staged_piece = random_next_piece().center(width);

        Self {
            height,
            width,
            cells,
            current_piece,
            staged_piece,
            drop_counter: 0,
            gameover: false,
            score: 0,
            hard_drop_count: 0,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width * CELL_SIZE, self.height * CELL_SIZE)
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
        let mut hard_drop_count = 0;
        while !self.move_piece_down() {
            hard_drop_count += 1;
        }
        self.hard_drop_count = hard_drop_count;
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
        self.render_outline(renderer);

        // Render occupied cells on the board
        for cell in self.grid_iterator() {
            let idx = self.cell_index(cell);
            match self.cells[idx] {
                Brick::Occupied(brick_type) => {
                    let image = Image::from_brick_type(brick_type);
                    renderer.render_image(image, cell, Opacity::Opaque);
                }
                Brick::Breaking(frame) => {
                    let image = Image::from_brick_type(BrickType::Smoke(frame));
                    renderer.render_image(image, cell, Opacity::Opaque);
                }
                _ => {}
            }
        }

        self.render_staged_piece(renderer);
        self.render_piece(renderer, &self.current_piece, Opacity::Opaque);
        self.render_piece(renderer, &self.ghost_piece(), Opacity::Translucent(128));
        self.render_score(renderer);
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
        if self.does_piece_fit(&self.staged_piece) {
            self.current_piece = self.staged_piece;
            self.staged_piece = random_next_piece().center(self.width);
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
        let mut number_lines_cleared = 0;
        for MatchingLine { cells, .. } in self.lines_matching(|_, brick| !brick.is_empty()) {
            number_lines_cleared += 1;
            for cell in cells {
                let idx = self.cell_index(cell);
                self.cells[idx] = Brick::Breaking(0);
            }
        }
        self.add_score(number_lines_cleared);
    }

    fn add_score(&mut self, number_lines_cleared: u32) {
        // https://tetris.wiki/Scoring#Original_BPS_scoring_system

        // if nothing has been cleared, no score should be added
        if number_lines_cleared == 0 {
            return;
        }

        let points = match number_lines_cleared {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => unreachable!("This case should never occur"),
        };

        // only add hard_drop_points if we've been hard dropped
        let hard_drop_points = match self.hard_drop_count {
            0 => 0,
            _ => self.hard_drop_count + 1,
        };

        self.score += points + hard_drop_points;

        // reset hard_drop_count for next piece
        self.hard_drop_count = 0;
    }

    fn move_bricks_down(&mut self, line: i32) {
        for row in (0..line).rev() {
            for col in 0..self.width {
                let cell = GridCell {
                    col: col as i32,
                    row,
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
    }

    fn ghost_piece(&self) -> Piece {
        let mut ghost_piece = self.current_piece;
        let mut next_ghost_piece = ghost_piece.move_down();

        while self.does_piece_fit(&next_ghost_piece) {
            ghost_piece = next_ghost_piece;
            next_ghost_piece = ghost_piece.move_down();
        }

        ghost_piece
    }

    fn render_piece(&self, renderer: &mut Renderer, piece: &Piece, opacity: Opacity) {
        for cell in piece.global_iter() {
            renderer.render_image(piece.image(), cell, opacity);
        }
    }

    fn render_staged_piece(&self, renderer: &mut Renderer) {
        let bg_color = Color::RGB(44, 44, 44);

        renderer.with_relative_offset(0, -(CELL_SIZE as i32 * 5), |renderer| {
            let bg_width = self.width * CELL_SIZE;
            renderer.fill_rect(Rect::new(0, 0, bg_width, 5 * CELL_SIZE), bg_color);

            renderer.render_text(
                Text::new("Next Block")
                    .height(20)
                    .center_xy((bg_width / 2) as i32, 10)
                    .build(),
            );

            self.render_piece(renderer, &self.staged_piece.move_down(), Opacity::Opaque);
        });
    }

    fn render_score(&self, renderer: &mut Renderer) {
        let bg_color = Color::RGB(44, 44, 44);

        renderer.with_relative_offset(0, (CELL_SIZE * self.height) as i32, |renderer| {
            let bg_width = CELL_SIZE * self.width;
            renderer.fill_rect(Rect::new(0, 0, bg_width, CELL_SIZE * 2), bg_color);

            let score_text = format!("Score: {}", self.score);
            renderer.render_text(
                Text::from(score_text)
                    .height(20)
                    .left_top_xy(10, 10)
                    .build(),
            );
        });
    }

    fn render_outline(&self, renderer: &mut Renderer) {
        let bg_color = Color::RGB(22, 22, 22);
        let stripe_color = Color::RGB(36, 36, 36);

        renderer.fill_rect(
            Rect::new(0, 0, self.width * CELL_SIZE, self.height * CELL_SIZE),
            bg_color,
        );

        for i in 0..self.width {
            let x = i * CELL_SIZE;

            renderer.fill_rect(
                Rect::new(x as i32, 0, 1, self.height * CELL_SIZE),
                stripe_color,
            );
        }

        for i in 0..self.height {
            let y = i * CELL_SIZE;

            renderer.fill_rect(
                Rect::new(0, y as i32, self.width * CELL_SIZE, 1),
                stripe_color,
            );
        }
    }
}
