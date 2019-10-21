use std::convert::From;
use std::ops::Add;

// --------
// GridCell
// --------

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GridCell {
    pub col: i32,
    pub row: i32,
}

impl GridCell {
    pub fn in_bounds(&self, width: i32, height: i32) -> bool {
        self.col >= 0 && self.col <= width - 1 && self.row >= 0 && self.row <= height - 1
    }
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

pub struct BrickIterator {
    origin: (i32, i32),
    num_columns: u32,
    num_rows: u32,
    current_col: i32,
    current_row: i32,
    cells: Vec<bool>,
}

impl BrickIterator {
    pub fn new(origin: (i32, i32), num_columns: u32, num_rows: u32, cells: Vec<bool>) -> Self {
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

                if self.cells[index] {
                    let (col_offset, row_offset) = self.origin;
                    let col = self.current_col + col_offset;
                    let row = self.current_row + row_offset;

                    self.current_col += 1;
                    return Some(GridCell { col, row });
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
