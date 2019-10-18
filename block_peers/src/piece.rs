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

    /// `local_iter` provides the (col, row) of occupied bricks inside the "local"
    /// 4x4 context of a PieceShape taking the current rotation into consideration.
    pub fn local_iter(&self) -> PieceIterator {
        PieceIterator::new(self.cells(), self.rotation, None)
    }

    /// `global_iter` provides the (col, row) of occupied bricks inside the "global"
    /// grid context of a PieceShape taking the current rotation and the pieces
    /// origin into consideration.
    #[allow(dead_code)]
    pub fn global_iter(&self) -> PieceIterator {
        PieceIterator::new(self.cells(), self.rotation, Some(self.position))
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.position.col, self.position.row)
    }
}

impl Piece {
    fn new(shape_idx: usize) -> Self {
        if shape_idx > SHAPES.len() {
            panic!("tried providing a piece shape index that doesn't exist");
        }

        Self {
            shape_idx,
            rotation: Rotation::Zero,
            position: GridCell { col: 0, row: 0 },
        }
    }

    fn cells(&self) -> Vec<bool> {
        SHAPES[self.shape_idx].to_vec()
    }
}

pub struct PieceIterator {
    current_col: usize,
    current_row: usize,
    rotation: Rotation,
    cells: Vec<bool>,
    position: Option<GridCell>,
}

impl PieceIterator {
    fn new(cells: Vec<bool>, rotation: Rotation, position: Option<GridCell>) -> Self {
        Self {
            current_col: 0,
            current_row: 0,
            rotation,
            cells,
            position,
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
                    let col = self.current_col;
                    let row = self.current_row;

                    self.current_col += 1;

                    if let Some(pos) = self.position {
                        return Some((col + pos.col as usize, row + pos.row as usize).into());
                    } else {
                        return Some((col, row).into());
                    }
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
enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    fn next(&self) -> Rotation {
        use Rotation::*;
        match self {
            Zero => Ninety,
            Ninety => OneEighty,
            OneEighty => TwoSeventy,
            TwoSeventy => Zero,
        }
    }
}

fn rotated_index(px: usize, py: usize, rotation: Rotation) -> usize {
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

#[test]
#[should_panic]
fn test_piece_indexing() {
    Piece::new(10);
}

#[test]
fn test_piece_iterator() {
    struct PieceTest {
        piece: Piece,
        expected: Vec<GridCell>,
    }
    let tests: Vec<PieceTest> = vec![
        PieceTest {
            piece: Piece::new(0),
            expected: vec![(1, 0).into(), (2, 0).into(), (3, 0).into(), (2, 1).into()],
        },
        PieceTest {
            piece: Piece::new(1),
            expected: vec![(2, 0).into(), (2, 1).into(), (2, 2).into(), (2, 3).into()],
        },
        PieceTest {
            piece: Piece::new(2),
            expected: vec![(1, 1).into(), (2, 1).into(), (1, 2).into(), (2, 2).into()],
        },
        PieceTest {
            piece: Piece::new(3),
            expected: vec![(0, 1).into(), (1, 1).into(), (1, 2).into(), (2, 2).into()],
        },
        PieceTest {
            piece: Piece::new(4),
            expected: vec![(1, 0).into(), (1, 1).into(), (2, 1).into(), (2, 2).into()],
        },
        PieceTest {
            piece: Piece::new(5),
            expected: vec![(2, 0).into(), (2, 1).into(), (1, 2).into(), (2, 2).into()],
        },
        PieceTest {
            piece: Piece::new(6),
            expected: vec![(1, 0).into(), (1, 1).into(), (1, 2).into(), (2, 2).into()],
        },
    ];

    for tt in tests {
        let mut result: Vec<GridCell> = vec![];
        for cell in tt.piece.local_iter() {
            result.push(cell);
        }
        assert_eq!(tt.expected, result);
    }
}
