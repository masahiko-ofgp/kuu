use crate::buff::Buffer;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::highlight::Highlighter;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyBindMode {
    Vim,
    Emacs,
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Normal,
    Insert,
    Command,
    Quit,
}

pub struct App {
    pub mode: AppMode,
    pub buffer: Buffer,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub row_offset: usize,
    pub file_path: Option<PathBuf>,
    pub config: Config,
    pub command_input: String,
    pub highlighter: Highlighter,
    pub status_message: Option<String>,
    pub pending_cmd: Option<char>,
    pub yank_register: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Normal,
            buffer: Buffer::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_offset: 0,
            file_path: None,
            config: Config::default(),
            command_input: String::new(),
            highlighter: Highlighter::new(),
            status_message: None,
            pending_cmd: None,
            yank_register: None,
        }
    }

    pub fn with_file(mut self, path: PathBuf) -> Self {
        match Buffer::load(&path) {
            Ok(buffer) => {
                self.buffer = buffer;
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    self.highlighter.set_language_by_extension(ext);
                }
                self.file_path = Some(path);
            }
            Err(_) => {
                self.file_path = Some(path);
            }
        }
        self
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            match self.buffer.save(path) {
                Ok(_) => {
                    self.status_message = Some(format!("Written to {:?}", path));
                }
                Err(e) => {
                    self.status_message = Some(format!("Error saving: {}", e));
                }
            }
        
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                self.highlighter.set_language_by_extension(ext);
            }
        }
        Ok(())
    }

    pub fn open(&mut self, path: PathBuf) {
        if let Ok(buffer) = Buffer::load(&path) {
            self.buffer = buffer;

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                self.highlighter.set_language_by_extension(ext);
            }

            self.file_path = Some(path);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let char_count = self.buffer.lines[self.cursor_y].chars().count();
        if self.cursor_x < char_count {
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
        let char_count = self.buffer.lines[self.cursor_y].chars().count();
        if self.cursor_x > char_count {
            self.cursor_x = char_count;
        }
    }

    pub fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.cursor_y, self.cursor_x);
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_x > 0 {
            self.buffer.delete_char(self.cursor_y, self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            let prev_y = self.cursor_y - 1;
            if let Some(join_point) = self.buffer.join_lines(prev_y) {
                self.cursor_y = prev_y;
                self.cursor_x = join_point;
            }
        }
    }

    pub fn open_new_line_below(&mut self) {
        self.buffer.insert_empty_line(self.cursor_y);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.mode = AppMode::Insert;
    }

    pub fn open_new_line_above(&mut self) {
        self.buffer.insert_line_above(self.cursor_y);
        self.cursor_x = 0;
        self.mode = AppMode::Insert;
    }

    pub fn yank_current_line(&mut self) {
        if let Some(line) = self.buffer.get_line(self.cursor_y) {
            self.yank_register = Some(line);
            self.status_message = Some("Yanked 1 line".to_string());
        }
    }

    pub fn put_after(&mut self) {
        if let Some(text) = &self.yank_register {
            self.buffer.insert_line(self.cursor_y, text.clone());
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.status_message = Some("Pasted".to_string());
        }
    }

    pub fn put_before(&mut self) {
        if let Some(text) = &self.yank_register {
            self.buffer.insert_line_at(self.cursor_y, text.clone());
            self.cursor_x = 0;
            self.status_message = Some("Pasted above".to_string());
        }
    }

    pub fn delete_current_line(&mut self) {
        if let Some(line) = self.buffer.get_line(self.cursor_y) {
            self.yank_register = Some(line);
        }
        let new_len = self.buffer.delete_line(self.cursor_y);
        if self.cursor_y >= new_len && self.cursor_y > 0 {
            self.cursor_y = new_len - 1;
        }
        self.snap_cursor_to_line_end();
    }
    pub fn scroll(&mut self, terminal_height: usize) {
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }

        if self.cursor_y >= self.row_offset + terminal_height {
            self.row_offset = self.cursor_y - terminal_height + 1;
        }
    }

    pub fn show_status_message(&self) -> String {
        match &self.status_message {
            Some(s) => s.to_string(),
            None => "NO MESSAGE".to_string(),
        }
    }
}
