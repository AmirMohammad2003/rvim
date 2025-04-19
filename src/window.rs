use std::{
    char,
    cmp::min,
    sync::{Arc, Mutex},
};

use crate::{buffer, cursor};

pub struct Window {
    pub buffer: Arc<Mutex<buffer::Buffer>>,
    pub cursor: cursor::Cursor,
    pub topline: usize,
    pub leftcol: usize,
    pub width: usize,
    pub height: usize,
}

impl Window {
    pub fn new(buffer: Arc<Mutex<buffer::Buffer>>, width: usize, height: usize) -> Self {
        let cursor = cursor::Cursor { col: 0, row: 0 };
        Self {
            buffer,
            cursor,
            topline: 0,
            leftcol: 0,
            width,
            height,
        }
    }

    fn fix_col(&mut self) {
        let linenr = self.cursor.row + self.topline;
        let linelen = self.buffer.lock().unwrap().content[linenr].len();
        if self.cursor.col >= linelen {
            self.cursor.col = linelen;
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor.row == 0 {
            if self.topline > 0 {
                self.topline -= 1;
                self.fix_col();
            }
        } else {
            self.cursor.move_up();
            self.fix_col();
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor.row + self.topline + 1 != self.buffer.lock().unwrap().lines {
            if self.cursor.row + 1 == self.height {
                self.topline += 1;
                self.fix_col();
            } else {
                self.cursor.move_down(self.height);
                self.fix_col();
            }
        }
    }

    pub fn cursor_left(&mut self) {
        self.cursor.move_left();
    }

    pub fn cursor_right(&mut self) {
        let linenr = self.cursor.row + self.topline;
        let linelen = self.buffer.lock().unwrap().content[linenr].len();
        self.cursor.move_right(linelen);
    }

    pub fn cursor_move_to(&mut self, row: usize, col: usize) {
        self.cursor.move_to(row, col);
        self.fix_col();
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn type_char(&mut self, ch: char) {
        let mut newline = false;
        if ch == '\n' {
            newline = true;
        }

        let linenr = self.cursor.row + self.topline;
        {
            let mut buffer = self.buffer.lock().unwrap();
            let linelen = buffer.content[linenr].len();
            if self.cursor.col >= linelen {
                if newline {
                    buffer.content.insert(linenr + 1, String::new());
                    buffer.lines += 1;
                } else {
                    buffer.content[linenr].push(ch);
                }
            } else if newline {
                let new_line = buffer.content[linenr].split_off(self.cursor.col);
                buffer.content.insert(linenr + 1, new_line);
                buffer.lines += 1;
            } else {
                buffer.content[linenr].insert(self.cursor.col, ch);
            }
            buffer.edited = true;
        }
        if newline {
            self.cursor_down();
            self.cursor.col = 0;
        } else {
            self.cursor_right();
        }
    }

    pub fn backspace(&mut self) {
        let linenr = self.cursor.row + self.topline;
        let mut left = false;
        {
            let mut buffer = self.buffer.lock().unwrap();
            let linelen = buffer.content[linenr].len();
            if self.cursor.col >= linelen {
                if linelen == 0 {
                    if (buffer.lines == 1) || (linenr == 0) {
                        return;
                    }
                    buffer.content.remove(linenr);
                    buffer.lines -= 1;
                    let length = buffer.content[linenr - 1].len();
                    self.cursor.col = length;
                } else {
                    buffer.content[linenr].pop();
                    left = true;
                }
            } else if self.cursor.col == 0 {
                if linenr == 0 {
                    return;
                }
                let line = buffer.content.remove(linenr);
                buffer.lines -= 1;
                let length = buffer.content[linenr - 1].len();
                buffer.content[linenr - 1].push_str(&line);
                self.cursor.col = length;
            } else {
                buffer.content[linenr].remove(self.cursor.col - 1);
                left = true;
            }
            buffer.edited = true;
        }
        if left {
            self.cursor_left();
        } else {
            self.cursor_up();
        }
    }

    pub fn visible_range(&self) -> (usize, usize, usize, usize, usize) {
        let end = min(
            self.topline + self.height,
            self.buffer.lock().unwrap().lines,
        );
        let top_visible = end - self.topline;
        let filler = self.height - top_visible;

        (
            self.topline,
            end,
            self.leftcol,
            self.leftcol + self.width,
            filler,
        )
    }

    pub fn set_buffer(&mut self, buffer: Arc<Mutex<buffer::Buffer>>) {
        self.buffer = buffer;
        self.topline = 0;
        self.leftcol = 0;
        self.cursor.row = 0;
        self.cursor.col = 0;
    }
}
