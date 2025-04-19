#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
}

impl Cursor {
    pub fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        } else {
            self.row = 0;
        }
    }

    pub fn move_down(&mut self, max_lines: usize) {
        if self.row + 1 < max_lines {
            self.row += 1;
        } else {
            self.row = max_lines - 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else {
            self.col = 0;
        }
    }

    pub fn move_right(&mut self, max_columns: usize) {
        if self.col < max_columns {
            self.col += 1;
        } else {
            self.col = max_columns;
        }
    }

    pub fn move_to(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
}
