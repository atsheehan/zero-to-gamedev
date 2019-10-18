use rand::Rng;

use crate::brick::GridCell;

// --------
// Piece
// --------

type PieceShape = [bool; 16];

#[cfg_attr(rustfmt, rustfmt_skip)]
const SHAPES: [PieceShape; 7] = [
    [
        false, true,  true,  true, 
        false, false, true,  false, 
        false, false, false, false, 
        false, false, false, false,
    ],
    [
        false, false, true, false,
        false, false, true, false,
        false, false, true, false,
        false, false, true, false
    ],
    [
        false, false, false, false,
        false, true,  true,  false,
        false, true,  true,  false,
        false, false, false, false
    ],
    [
        false, false, false, false,
        true,  true,  false, false,
        false, true,  true,  false,
        false, false, false, false
    ],
    [
        false,  true, false, false,
        false,  true, true,  false,
        false, false, true,  false,
        false, false, false, false
    ],
    [
        false, false,  true, false,
        false, false,  true, false,
        false,  true,  true, false,
        false, false, false, false
    ],
    [
        false,  true, false, false,
        false,  true, false, false,
        false,  true, true,  false,
        false, false, false, false
    ],
];

pub fn random_next_piece() -> Piece {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0, SHAPES.len());
    Piece::new(idx)
}

pub struct Piece {
    shape_idx: usize,
    rotation: Rotation,
    position: GridCell,
}

impl Piece {
    fn new(shape_idx: usize) -> Self {
        Self {
            shape_idx,
            rotation: Rotation::Zero,
            position: GridCell { col: 0, row: 0 },
        }
    }

    // TODO: Temporary until we refactor Piece out
    fn cells(&self) -> Vec<bool> {
        SHAPES[self.shape_idx].to_vec()
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.position.col, self.position.row)
    }

    pub fn rotate(&self) -> Self {
        Self {
            shape_idx: self.shape_idx,
            rotation: self.rotation.next(),
            position: self.position.clone(),
        }
    }

    pub fn move_down(&self) -> Self {
        Self {
            shape_idx: self.shape_idx,
            rotation: self.rotation,
            position: self.position + (0, 1),
        }
    }

    pub fn move_right(&self) -> Self {
        Self {
            shape_idx: self.shape_idx,
            rotation: self.rotation,
            position: self.position + (1, 0),
        }
    }

    pub fn move_left(&self) -> Self {
        Self {
            shape_idx: self.shape_idx,
            rotation: self.rotation,
            position: self.position + (-1, 0),
        }
    }

    pub fn piece_iter(&self) -> PieceIterator {
        PieceIterator::new(self.cells(), self.rotation)
    }
}

pub struct PieceIterator {
    current_col: usize,
    current_row: usize,
    rotation: Rotation,
    cells: Vec<bool>,
}

impl PieceIterator {
    fn new(cells: Vec<bool>, rotation: Rotation) -> Self {
        Self {
            current_col: 0,
            current_row: 0,
            rotation,
            cells,
        }
    }
}

impl Iterator for PieceIterator {
    type Item = GridCell;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_row < 4 {
            while self.current_col < 4 {
                let index = rotated_index(self.current_col, self.current_row, self.rotation);
                if self.cells[index] {
                    self.current_col += 1;
                    return Some((self.current_col, self.current_row).into());
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

// --------
// Rotation
// --------

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub fn next(&self) -> Rotation {
        use Rotation::*;
        match self {
            Zero => Ninety,
            Ninety => OneEighty,
            OneEighty => TwoSeventy,
            TwoSeventy => Zero,
        }
    }
}

pub fn rotated_index(px: usize, py: usize, rotation: Rotation) -> usize {
    use Rotation::*;

    match rotation {
        Zero => py * 4 + px,
        Ninety => 12 + py - (px * 4),
        OneEighty => 15 - (py * 4) - px,
        TwoSeventy => 3 - py + (px * 4),
    }
}

// --------
// Tests
// --------

#[test]
fn test_next_rotation() {
    assert_eq!(Rotation::Zero.next(), Rotation::Ninety);
    assert_eq!(Rotation::Ninety.next(), Rotation::OneEighty);
    assert_eq!(Rotation::OneEighty.next(), Rotation::TwoSeventy);
    assert_eq!(Rotation::TwoSeventy.next(), Rotation::Zero);
}
