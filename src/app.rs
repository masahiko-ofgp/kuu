use crate::buff::Buffer;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::highlight::Highlighter;
use crate::history::{HistoryManager, EditAction};
use std::fs;


#[derive(PartialEq)]
enum CharKind {
    Whitespace,
    Word,
    Punctuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyBindMode {
    Vim,
    Emacs,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConfirmAction {
    Quit,
    OpenFile(PathBuf),
    CloseFile,
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Normal,
    Insert,
    Command,
    FileTree,
    Confirm,
    Help,
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
    pub pending_confirm_action: Option<ConfirmAction>,
    pub pending_cmd: Option<char>,
    pub yank_register: Option<String>,
    pub file_list: Vec<PathBuf>,
    pub file_list_selected: usize,
    pub file_tree_offset: usize,
    pub show_file_tree: bool,
    pub current_dir: PathBuf,
    pub editor_viewport_height: u16,
    pub file_viewport_height: u16,
    pub history: HistoryManager,
    pub help_scroll_offset: usize,
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
            pending_confirm_action: None,
            pending_cmd: None,
            yank_register: None,
            file_list: Vec::new(),
            file_list_selected: 0,
            file_tree_offset: 0,
            show_file_tree: true,
            current_dir: current_dir.clone(),
            editor_viewport_height: 0,
            file_viewport_height: 0,
            history: HistoryManager::new(),
            help_scroll_offset: 0,
        };

        app.update_file_list(current_dir);
        app
    }


    // ========= Config ============


    pub fn open_config(&mut self) {
        let path = self.config.get_config_path();

        if self.is_buffer_modified() {
            self.status_message = Some("Save current changes first!".to_string());
            return;
        }

        if path.exists() {
            self.open(path);
            self.status_message = Some("Editing config.toml".to_string());
        } else {
            self.status_message = Some("config.toml not found, Create it first?".to_string());
        }
    }

    pub fn reload_config(&mut self) {
        let config_path = self.config.get_config_path();

        let is_config = self.file_path.as_ref()
            .map(|p| p.canonicalize().ok() == config_path.canonicalize().ok())
                .unwrap_or(false);

        if is_config {
            let content = self.buffer.as_full_text();

            match toml::from_str::<Config>(&content) {
                Ok(new_config) => {
                    self.config = new_config;
                    self.status_message = Some("Config reloaded and applied!".to_string());
                }
                Err(e) => {
                    self.status_message = Some(format!("Config Error: {}", e));
                }
            }
        }
    }

    // ========= Help ==========

    pub fn show_help(&mut self) {
        self.mode = AppMode::Help;
        self.help_scroll_offset = 0;
        self.status_message = Some("Help Mode: Press Esc or q to close".to_string());
    }

    pub fn get_help_content(&self) -> Vec<String> {
        let content = match self.config.key_bind_mode {
            KeyBindMode::Vim => include_str!("../assets/help_vim.txt"),
            KeyBindMode::Emacs => include_str!("../assets/help_emacs.txt"),
        };
        content.lines()
            .map(|s| s.to_string())
            .collect()
    }

    // ========= Confirm ============


    pub fn request_confirm(&mut self, message: &str, action: ConfirmAction)
    {
        self.status_message = Some(format!("{} (y/n): ", message));
        self.pending_confirm_action = Some(action);
        self.mode = AppMode::Confirm;
    }

    pub fn confirm_action(&mut self) {
        if let Some(action) = self.pending_confirm_action.take() {
            match action {
                ConfirmAction::Quit => self.mode = AppMode::Quit,
                ConfirmAction::OpenFile(path) => {
                    self.open(path);
                    self.mode = AppMode::Normal;
                }
                ConfirmAction::CloseFile => {
                    self.buffer = Buffer::new();
                    self.file_path = None;
                    self.cursor_y = 0;
                    self.cursor_x = 0;
                    self.mode = AppMode::Normal;
                }
            }
            self.status_message = None;
        }
    }

    pub fn cancel_confirm(&mut self) {
        self.pending_confirm_action = None;
        self.status_message = Some("Canceled".to_string());
        self.mode = AppMode::Normal;
    }


    // ========= File Tree ===========


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
        }
        self.file_list_selected = 0;
        self.file_tree_offset = 0;
    }

    pub fn file_tree_next(&mut self) {
        if self.file_list_selected < self.file_list.len().saturating_sub(1) {
            self.file_list_selected += 1;
            self.scroll_tree();
        }
    }

    pub fn file_tree_prev(&mut self) {
        if self.file_list_selected > 0 {
            self.file_list_selected -= 1;
            self.scroll_tree();
        }
    }

    pub fn file_tree_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();
            self.update_file_list(parent_path);
        }
    }

    pub fn file_tree_select(&mut self) {
        if let Some(path) = self.file_list.get(self.file_list_selected).clone() {
            if path.is_dir() {
                self.update_file_list(path.to_path_buf());
            } else {
                if self.is_buffer_modified() {
                    self.request_confirm(
                        "Discord unsaved changes?",
                        ConfirmAction::OpenFile(path.to_path_buf())
                        );
                } else {
                    self.open(path.to_path_buf());
                    self.mode = AppMode::Normal;
                    self.status_message = Some("File opend".to_string());
                }
            }
        }
    }

    pub fn scroll_tree(&mut self) {
        let height = self.file_viewport_height as usize;

        if height == 0 { return; }

        if self.file_list_selected < self.file_tree_offset {
            self.file_tree_offset = self.file_list_selected;
        }

        if self.file_list_selected >= self.file_tree_offset + height {
            self.file_tree_offset = self.file_list_selected - height + 1;
        }
    }

    // =========== File open, close, save ============

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

    pub fn save_and_reload(&mut self) {
        let _ = self.save();
        self.reload_config();
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
            self.history.clear();
        }
    }

    pub fn execute_close_file(&mut self) {
        self.buffer = Buffer::new();
        self.file_path = None;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_offset = 0;
        self.highlighter = Highlighter::new();
        self.status_message = Some("File closed".to_string());
    }

    pub fn close_file(&mut self) {
        if self.is_buffer_modified() {
            self.request_confirm(
                "Discord unsaved changes?",
                ConfirmAction::CloseFile,
                );
        } else {
            self.execute_close_file();
        }
    }


    // ======= Cursor, Scroll ============


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

    pub fn center_cursor(&mut self) {
        let h = self.editor_viewport_height;

        if h > 0 {
            self.row_offset = self.cursor_y.saturating_sub((h / 2) as usize);
        }
    }

    pub fn move_word_forward(&mut self) {
        if let Some(line) = self.buffer.lines.get(self.cursor_y) {
            let chars: Vec<char> = line.chars().collect();
            let mut x = self.cursor_x;

            if x >= chars.len() {
                self.move_to_next_line_start();
                return;
            }

            let start_kind = self.get_char_kind(chars[x]);

            while x < chars.len() && self.get_char_kind(chars[x]) == start_kind { x += 1; }

            while x < chars.len() && chars[x].is_whitespace() { x += 1; }

            if x < chars.len() {
                self.cursor_x = x;
            } else {
                self.move_to_next_line_start();
            }
            self.snap_cursor();
        }
    }

    pub fn move_word_backward(&mut self) {
        if self.cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;

                let len = self.buffer.lines[self.cursor_y].chars().count();
                self.cursor_x = len.saturating_sub(1);
            }
            return;
        }

        if let Some(line) = self.buffer.lines.get(self.cursor_y) {
            let chars: Vec<char> = line.chars().collect();
            let mut x = self.cursor_x.saturating_sub(1);

            while x > 0 && chars[x].is_whitespace() {
                x -= 1;
            }

            let kind = self.get_char_kind(chars[x]);

            while x > 0 && self.get_char_kind(chars[x - 1]) == kind {
                x -= 1;
            }

            self.cursor_x = x;
        }
        self.snap_cursor();
    }

    fn move_to_next_line_start(&mut self) {
        if self.cursor_y < self.buffer.lines.len() - 1 {
            self.cursor_y += 1;
            let next_line = &self.buffer.lines[self.cursor_y];
            self.cursor_x = next_line
                .char_indices()
                .find(|(_, c)| !c.is_whitespace())
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn scroll_half_page_down(&mut self) {
        let h = self.editor_viewport_height;
        let amount = h / 2;

        for _ in 0..amount {
            self.move_cursor_down();
        }

        self.row_offset = self.row_offset.saturating_add(amount.into());
    }

    pub fn scroll_half_page_up(&mut self) {
        let h = self.editor_viewport_height as usize;
        let amount = h / 2;

        for _ in 0..amount {
            self.move_cursor_up();
        }

        self.row_offset = self.row_offset.saturating_sub(amount);
    }

    pub fn update_viewport_height(&mut self, height: u16) {
        self.editor_viewport_height = height;
    }

    pub fn scroll(&mut self, terminal_height: usize) {
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }

        if self.cursor_y >= self.row_offset + terminal_height {
            self.row_offset = self.cursor_y - terminal_height + 1;
        }
    }


    // ========== Insert, Delete ===========


    pub fn replace_char(&mut self, c: char) {
        self.delete_char();
        self.insert_char(c);
    }

    pub fn insert_char(&mut self, c: char) {
        let line = self.cursor_y;
        let col = self.cursor_x;
        self.buffer.insert_char(line, col, c);
        self.history.push_undo(EditAction::InsertChar { line, col, c });
        self.cursor_x += 1;
    }

    pub fn insert_newline(&mut self) {
        let line = self.cursor_y;
        let col = self.cursor_x;
        self.buffer.split_line(line, col);
        self.history.push_undo(EditAction::InsertNewline { line, col });
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    pub fn insert_tab(&mut self) {
        let tab_size = self.config.tab_size;
        for _ in 0..tab_size {
            self.insert_char(' ');
            self.cursor_x += 1;
        }
    }

    pub fn delete_char(&mut self) {
        let line = self.cursor_y;
        let col = self.cursor_x;

        if let Some(c) = self.buffer.get_char(line, col) {
            self.buffer.delete_char(line, col);
            self.history.push_undo(EditAction::DeleteChar { line, col, c });
        } else if self.cursor_y < self.buffer.lines.len() - 1 {
            if let Some(join_point) = self.buffer.join_lines(line) {
                self.history.push_undo(EditAction::DeleteNewline { line, col: join_point });
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_x > 0 {
            let line = self.cursor_y;
            let col = self.cursor_x - 1;
            if let Some(c) = self.buffer.get_char(line, col) {
                self.buffer.delete_char(line, col);
                self.history.push_undo(EditAction::DeleteChar { line, col, c });
                self.cursor_x = col;
            }
        } else if self.cursor_y > 0 {
            let target_line = self.cursor_y - 1;
            if let Some(join_point) = self.buffer.join_lines(target_line) {
                self.history.push_undo(EditAction::DeleteNewline { line: target_line, col: join_point });
                self.cursor_y = target_line;
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
        let line = self.cursor_y;
        if let Some(text) = self.buffer.lines.get(line).cloned() {
            self.buffer.delete_line(line);
            self.history.push_undo(EditAction::DeleteLine { line, text });
            self.cursor_y = line.min(self.buffer.lines.len() - 1);
            self.snap_cursor();
        }
    }

    pub fn change_current_line(&mut self) {
        if let Some(line) = self.buffer.lines.get(self.cursor_y) {
            self.yank_register = Some(line.clone());

            let indent: String = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();

            let indent_len = indent.chars().count();

            if let Some(line_mut) = self.buffer.lines.get_mut(self.cursor_y) {
                line_mut.clear();
                line_mut.push_str(&indent);
                self.buffer.mark_dirty();
            }

            self.cursor_x = indent_len;
            self.mode = AppMode::Insert;
        }
        self.snap_cursor();
    }

    pub fn change_to_end_of_line(&mut self) {
        let line = self.cursor_y;
        let col = self.cursor_x;
        self.history.start_group();
        let tail = self.buffer.truncate_line(line, col);
        for (i, c) in tail.chars().enumerate() {
            self.history.push_undo(EditAction::DeleteChar {
                line,
                col: col + i,
                c
            });
        }
        self.yank_register = Some(tail);
        self.mode = AppMode::Insert;
    }

    pub fn indent_current_line(&mut self) {
        let tab_size = self.config.tab_size;
        let indent = " ".repeat(tab_size);
        self.history.start_group();
        for (i, c) in indent.chars().enumerate() {
            self.buffer.insert_char(self.cursor_y, i ,c);
            self.history.push_undo(EditAction::InsertChar { line: self.cursor_y, col: i, c});
        }
        self.history.finish_group();
        self.cursor_x = self.buffer.first_non_whitespace_idx(self.cursor_y);
        //self.snap_cursor();
    }

    pub fn unindent_current_line(&mut self) {
        let tab_size = self.config.tab_size;
        self.history.start_group();

        let spaces = self.buffer.remove_leading_spaces(self.cursor_y, tab_size);
        for _ in 0..spaces {
            self.history.push_undo(EditAction::DeleteChar {
                line: self.cursor_y,
                col: 0,
                c: ' '
            });
        }
        self.history.finish_group();
        self.cursor_x = self.buffer.first_non_whitespace_idx(self.cursor_y);
        //self.snap_cursor();
    }


    // ====== Undo, Redo ========


    pub fn undo(&mut self) {
        if let Some(action) = self.history.pop_undo() {
            self.apply_action_reverse(&action);
            self.history.push_redo(action);
            self.status_message = Some("Undo".to_string());
        } else {
            self.status_message = Some("Already at oldest change".to_string());
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.history.pop_redo() {
            let undo_action = action.reverse();
            self.apply_action(&action);
            self.history.push_undo_from_redo(undo_action);
            self.status_message = Some("Redo".to_string());
        } else {
            self.status_message = Some("Already at newest change".to_string());
        }
    }

    pub fn apply_action(&mut self, action: &EditAction) {
        match action {
            EditAction::InsertChar { line, col, c } => {
                self.buffer.insert_char(*line, *col, *c);
                self.cursor_y = *line;
                //self.cursor_x = *col;
                self.cursor_x = *col + 1;
            }
            EditAction::DeleteChar { line, col, .. } => {
                self.buffer.delete_char(*line, *col);
                self.cursor_y = *line;
                self.cursor_x = *col;
            }
            EditAction::InsertNewline { line, .. } => {
                self.insert_newline();
                self.cursor_y = *line + 1;
                self.cursor_x = 0;
            }
            EditAction::DeleteNewline { line, col } => {
                self.buffer.delete_line(*line);
                self.cursor_y = *line;
                self.cursor_x = *col;
            }
            EditAction::InsertLine { line, text } => {
                self.buffer.insert_line_at(*line, text.clone());
                self.cursor_y = *line;
            }
            EditAction::DeleteLine { line, .. } => {
                self.buffer.delete_line(*line);
                self.cursor_y = (*line).min(self.buffer.lines.len() - 1);
            }
            EditAction::Group(actions) => {
                for a in actions {
                    self.apply_action(a);
                }
            }
        }
        self.buffer.modified = true;
    }

    pub fn apply_action_reverse(&mut self, action: &EditAction) {
        match action {
            EditAction::InsertChar { line, col, .. } => {
                self.buffer.delete_char(*line, *col);
                self.cursor_y = *line;
                self.cursor_x = *col;
            }
            EditAction::DeleteChar { line, col, c } => {
                self.buffer.insert_char(*line, *col, *c);
                self.cursor_y = *line;
                self.cursor_x = *col + 1;
            }
            EditAction::InsertNewline { line, col } => {
                self.buffer.delete_line(*line);
                self.cursor_y = *line;
                self.cursor_x = *col;
            }
            EditAction::DeleteNewline { line, .. } => {
                self.insert_newline();
                self.cursor_y = *line + 1;
                self.cursor_x = 0;
            }
            EditAction::InsertLine { line, .. } => {
                self.buffer.delete_line(*line);
                self.cursor_y = (*line).min(self.buffer.lines.len() - 1);
            }
            EditAction::DeleteLine { line, text } => {
                self.buffer.insert_line_at(*line, text.clone());
                self.cursor_y = *line;
            }
            EditAction::Group(actions) => {
                for a in actions.iter().rev() {
                    self.apply_action_reverse(a);
                }
            }
        }
        self.buffer.modified = true;
    }

    // ===== Helper =======


    fn get_char_kind(&self, c: char) -> CharKind {
        if c.is_whitespace() {
            CharKind::Whitespace
        } else if c.is_alphanumeric() || c == '_' {
            CharKind::Word
        } else {
            CharKind::Punctuation
        }
    }

    pub fn is_buffer_modified(&self) -> bool {
        self.buffer.modified
    }

    pub fn clear_pending(&mut self) {
        self.pending_cmd = None;
    }

}
