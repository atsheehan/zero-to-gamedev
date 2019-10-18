use std::convert::From;
use std::ops::Add;

// --------
// GridCell
// --------

#[derive(Debug, Copy, Clone)]
pub struct GridCell {
    pub col: i32,
    pub row: i32,
}

impl From<(i32, i32)> for GridCell {
    fn from(item: (i32, i32)) -> Self {
        Self {
            col: item.0,
            row: item.1,
        }
    }
}

impl From<(usize, usize)> for GridCell {
    fn from(item: (usize, usize)) -> Self {
        Self {
            col: item.0 as i32,
            row: item.1 as i32,
        }
    }
}

impl Add<(i32, i32)> for GridCell {
    type Output = Self;

    fn add(self, rhs: (i32, i32)) -> Self {
        Self {
            col: self.col + rhs.0,
            row: self.row + rhs.1,
        }
    }
}

impl From<GridCell> for (i32, i32) {
    fn from(item: GridCell) -> Self {
        (item.col, item.row)
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
    only_occupied: bool,
}

impl BrickIterator {
    pub fn new(
        origin: (i32, i32),
        num_columns: u32,
        num_rows: u32,
        cells: Vec<bool>,
        only_occupied: bool,
    ) -> Self {
        BrickIterator {
            origin,
            num_columns,
            num_rows,
            current_col: 0,
            current_row: 0,
            cells,
            only_occupied,
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

                if self.only_occupied {
                    if self.cells[index] {
                        let (col_offset, row_offset) = self.origin;
                        let col = self.current_col + col_offset;
                        let row = self.current_row + row_offset;

                        self.current_col += 1;
                        return Some(GridCell { col, row });
                    } else {
                        self.current_col += 1;
                    }
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
