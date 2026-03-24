use crate::buff::Buffer;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::highlight::Highlighter;
use std::fs;


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
    FileTree,
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
    pub file_list: Vec<PathBuf>,
    pub file_list_selected: usize,
    pub show_file_tree: bool,
    pub current_dir: PathBuf,
}

impl App {
    pub fn new() -> Self {

        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        let mut app = Self {
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
            file_list: Vec::new(),
            file_list_selected: 0,
            show_file_tree: true,
            current_dir: current_dir.clone(),
        };

        app.update_file_list(current_dir);
        app
    }

    pub fn update_file_list(&mut self, path: PathBuf) {
        let target_dir = if let Ok(abs_path) = fs::canonicalize(&path) {
            if abs_path.is_dir() {
                abs_path
            } else {
                abs_path.parent()
                    .unwrap_or(&abs_path)
                    .to_path_buf()
            }
        } else {
            path
        };

        if let Ok(entries) = fs::read_dir(&target_dir) {
            self.current_dir = target_dir.clone();
            self.file_list.clear();

            if let Some(parent) = target_dir.parent() {
                self.file_list.push(parent.to_path_buf());
            }

            let mut files: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect();

            files.sort();

            self.file_list.extend(files);
            self.file_list_selected = 0;
        }
    }

    // TODO: This is placefolder.
    pub fn is_buffer_modified(&self) -> bool {
        false
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
            self.file_path = std::fs::canonicalize(&path)
                .ok()
                .or(Some(path));

            if let Some(ext) = self.file_path.as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str()) 
            {
                self.highlighter.set_language_by_extension(ext);
            }
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.row_offset = 0;
        }
    }

    pub fn close_file(&mut self) {
        self.buffer = Buffer::new();
        self.file_path = None;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_offset = 0;
        self.highlighter = Highlighter::new();
        self.status_message = Some("File closed".to_string());
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
            self.snap_cursor();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_y < self.buffer.lines.len() - 1 {
            self.cursor_y += 1;
            self.snap_cursor();
        }
    }

    pub fn snap_cursor(&mut self) {
        if let Some(line) = self.buffer.lines.get(self.cursor_y) {
            let char_count = line.chars().count();

            let max_x = if self.mode == AppMode::Insert {
                char_count
            } else {
                char_count.saturating_sub(1)
            };

            if self.cursor_x > max_x {
                self.cursor_x = max_x;
            }
        } else {
            self.cursor_y = self.buffer.lines.len()
                .saturating_sub(1);
            self.cursor_x = 0;
        }
    }

    pub fn insert_newline(&mut self) {
        self.buffer.split_line(self.cursor_y, self.cursor_x);
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
        self.buffer.insert_line_at(self.cursor_y + 1, String::new());
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.mode = AppMode::Insert;
    }

    pub fn open_new_line_above(&mut self) {
        self.buffer.insert_line_at(self.cursor_y, String::new());
        self.cursor_x = 0;
        self.mode = AppMode::Insert;
    }

    pub fn kill_line(&mut self) {
        let tail = self.buffer.truncate_line(self.cursor_y, self.cursor_x);

        if tail.is_empty() && self.cursor_y < self.buffer.lines.len() - 1 {
            self.buffer.join_lines(self.cursor_y);
        } else {
            self.yank_register = Some(tail);
        }
    }

    pub fn yank_current_line(&mut self) {
        if let Some(line) = self.buffer.lines.get(self.cursor_y).cloned() {
            self.yank_register = Some(line);
            self.status_message = Some("Yanked 1 line".to_string());
        }
    }

    pub fn put_after(&mut self) {
        if let Some(text) = &self.yank_register {
            self.buffer.insert_line_at(self.cursor_y, text.clone());
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
        if let Some(line) = self.buffer.lines.get(self.cursor_y).cloned() {
            self.yank_register = Some(line);
        }
        let new_len = self.buffer.delete_line(self.cursor_y);
        if self.cursor_y >= new_len && self.cursor_y > 0 {
            self.cursor_y = new_len - 1;
        }
        self.snap_cursor();
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
