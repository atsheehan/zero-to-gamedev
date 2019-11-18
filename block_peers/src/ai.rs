use rand::Rng;

use crate::grid::Grid;
use crate::render::Renderer;

/// DumbAI has no strategy for winning it will just randomly move pieces around
/// the board.
pub struct DumbAI {
    grid: Grid,
    move_counter: u32,
}

impl DumbAI {
    pub fn new(grid: Grid) -> Self {
        Self {
            grid,
            move_counter: 0,
        }
    }

    pub fn update(&mut self) {
        self.grid.update();

        self.move_counter += 1;

        if self.move_counter % 40 == 0 {
            self.move_counter = 0;
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0, 100);

            match idx {
                0..=10 => {
                    self.grid.move_piece_to_bottom();
                }
                11..=17 => {
                    self.grid.rotate();
                }
                18..=90 => {
                    self.grid.move_piece_right();
                }
                91..=100 => {
                    self.grid.move_piece_left();
                }
                _ => {}
            }
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        self.grid.render_for_title(renderer);
    }
}
