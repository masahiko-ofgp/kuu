use crate::buff::Buffer;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::Config;


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
}

impl App {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    pub fn with_file(path: PathBuf, config: Config) -> Self {
        let buffer = Buffer::load(&path)
            .unwrap_or_else(|_| Buffer::new());

        Self {
            mode: AppMode::Normal,
            buffer,
            cursor_x: 0,
            cursor_y: 0,
            row_offset: 0,
            file_path: Some(path),
            config,
            command_input: String::new(),
        }
    }
     pub fn with_config(config: Config) -> Self {
         Self {
             mode: AppMode::Normal,
             buffer: Buffer::new(),
             cursor_x: 0,
             cursor_y: 0,
             row_offset: 0,
             file_path: None,
             config,
             command_input: String::new(),
         }
     }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            self.buffer.save(path)?;
        }
        Ok(())
    }

    pub fn execute_command(&mut self) {
        let cmd = self.command_input.trim();
        
        match cmd {
            "w" | "write" => {
                let _ = self.save();
                self.mode = AppMode::Normal;
            },
            "q" | "quit" => {
                self.mode = AppMode::Quit;
            },
            "wq" => {
                let _ = self.save();
                self.mode = AppMode::Quit;
            },
            _ => {
                self.mode = AppMode::Normal;
            }
        }
        self.command_input.clear();
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

    pub fn scroll(&mut self, terminal_height: usize) {
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }

        if self.cursor_y >= self.row_offset + terminal_height {
            self.row_offset = self.cursor_y - terminal_height + 1;
        }
    }
}
