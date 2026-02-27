use crate::buff::Buffer;


#[derive(Debug, PartialEq)]
pub enum AppMode {
    Normal,
    Insert,
    Quit,
}

pub struct App {
    pub mode: AppMode,
    pub buffer: Buffer,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Normal,
            buffer: Buffer::new(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.buffer.lines[self.cursor_y].len();
        if self.cursor_x < line_len {
            self.cursor_x += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.snap_cursor_to_line_end();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_y < self.buffer.lines.len() - 1 {
            self.cursor_y += 1;
            self.snap_cursor_to_line_end();
        }
    }

    pub fn snap_cursor_to_line_end(&mut self) {
        let line_len = self.buffer.lines[self.cursor_y].len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

    pub fn insert_newline(&mut self) {
        self.buffer.insert_empty_line(self.cursor_y);
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_x > 0 {
            self.buffer.delete_char(self.cursor_y, self.cursor_x);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let prev_line_len = self.buffer.lines[self.cursor_y - 1].len();
            if let Some(new_x) = self.buffer.join_lines(self.cursor_y) {
                self.cursor_y -= 1;
                self.cursor_x = new_x;
            }
        }
    }

    pub fn open_new_line_below(&mut self) {
        self.buffer.insert_empty_line(self.cursor_y);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.mode = AppMode::Insert;
    }
}
