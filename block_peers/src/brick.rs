use serde::{Deserialize, Serialize};
use std::convert::From;
use std::ops::Add;

use crate::render::Image;

// --------
// GridCell
// --------

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct GridCell {
    pub col: i32,
    pub row: i32,
}

impl Default for GridCell {
    fn default() -> Self {
        Self { col: 0, row: 0 }
    }
}

impl From<(i32, i32)> for GridCell {
    fn from((col, row): (i32, i32)) -> Self {
        Self { col, row }
    }
}

impl From<(usize, usize)> for GridCell {
    fn from((col, row): (usize, usize)) -> Self {
        Self {
            col: col as i32,
            row: row as i32,
        }
    }
}

impl From<GridCell> for (i32, i32) {
    fn from(item: GridCell) -> Self {
        (item.col, item.row)
    }
}

impl Add<(i32, i32)> for GridCell {
    type Output = Self;

    fn add(self, (col, row): (i32, i32)) -> Self {
        Self {
            col: self.col + col,
            row: self.row + row,
        }
    }
}

impl Add<GridCell> for GridCell {
    type Output = Self;

    fn add(self, rhs: GridCell) -> Self {
        Self {
            col: self.col + rhs.col,
            row: self.row + rhs.row,
        }
    }
}

// -----
// Brick
// -----

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum BrickType {
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Purple,
    Teal,
    Smoke(u32),
}

/// Brick is used to represent the content in an (x, y) position on the
/// game grid.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Brick {
    Empty,
    Occupied(BrickType),
    Breaking(u32),
    Broken,
}

impl Brick {
    pub fn break_brick(self) -> Option<Brick> {
        match self {
            Brick::Breaking(frame) => {
                let next = frame + 1;
                if next < Image::max_smoke_frame() {
                    Some(Brick::Breaking(next))
                } else {
                    Some(Brick::Broken)
                }
            }
            _ => None,
        }
    }

    pub fn is_broken(&self) -> bool {
        match self {
            Brick::Broken => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Brick::Empty => true,
            _ => false,
        }
    }
}

pub struct BrickIterator {
    origin: (i32, i32),
    num_columns: u32,
    num_rows: u32,
    current_col: i32,
    current_row: i32,
    cells: Vec<Brick>,
}

impl BrickIterator {
    pub fn new(origin: (i32, i32), num_columns: u32, num_rows: u32, cells: Vec<Brick>) -> Self {
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
    type Item = GridCell;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_row < self.num_rows as i32 {
            while self.current_col < self.num_columns as i32 {
                let index =
                    ((self.current_row * self.num_columns as i32) + self.current_col) as usize;

                if self.cells[index] == Brick::Empty {
                    self.current_col += 1;
                } else {
                    let (col_offset, row_offset) = self.origin;
                    let col = self.current_col + col_offset;
                    let row = self.current_row + row_offset;

                    self.current_col += 1;
                    return Some(GridCell { col, row });
                }
            }

            self.current_row += 1;
            self.current_col = 0;
        }

        None
    }
}

pub struct MatchingLine {
    pub cells: Vec<GridCell>,
    pub row: u32,
}

pub struct LineIterator<CB>
where
    CB: Fn(GridCell, Brick) -> bool,
{
    cells: Vec<Brick>,
    num_columns: u32,
    num_rows: u32,
    current_col: i32,
    current_row: i32,
    callback: CB,
}

impl<CB> LineIterator<CB>
where
    CB: Fn(GridCell, Brick) -> bool,
{
    pub fn new(cells: Vec<Brick>, num_columns: u32, num_rows: u32, callback: CB) -> Self {
        Self {
            cells,
            num_columns,
            num_rows,
            current_col: 0,
            current_row: 0,
            callback,
        }
    }
}

impl<CB> Iterator for LineIterator<CB>
where
    CB: Fn(GridCell, Brick) -> bool,
{
    type Item = MatchingLine;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_row < self.num_rows as i32 {
            let mut all_true = true;
            let mut grid_cells = Vec::new();

            while self.current_col < self.num_columns as i32 {
                let index =
                    ((self.current_row * self.num_columns as i32) + self.current_col) as usize;

                let grid_cell = GridCell {
                    col: self.current_col,
                    row: self.current_row,
                };
                grid_cells.push(grid_cell);

                let brick = self.cells[index].clone();

                all_true &= (self.callback)(grid_cell, brick);
                self.current_col += 1;
            }

            let row = self.current_row as u32;
            self.current_row += 1;
            self.current_col = 0;

            if all_true {
                return Some(MatchingLine {
                    row,
                    cells: grid_cells,
                });
            }
        }

        None
    }
}
