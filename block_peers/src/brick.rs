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
    type Item = (i32, i32);

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
                    return Some((col, row));
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
