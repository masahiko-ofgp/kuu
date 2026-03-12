use crossterm::event::{KeyCode, KeyEvent};
use crate::app::{App, AppMode};
use super::KeyHandler;
use std::path::{PathBuf};


pub struct VimHandler;

impl KeyHandler for VimHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) {
        match app.mode {
            AppMode::Normal => self.handle_normal(key, app),
            AppMode::Insert => self.handle_insert(key, app),
            AppMode::Command => self.handle_command(key, app),
            _ => {}
        }
    }
}

impl VimHandler {
    fn handle_normal(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('i') => app.mode = AppMode::Insert,
            KeyCode::Char('h') => app.move_cursor_left(),
            KeyCode::Char('j') => app.move_cursor_down(),
            KeyCode::Char('k') => app.move_cursor_up(),
            KeyCode::Char('l') => app.move_cursor_right(),
            KeyCode::Char('o') => app.open_new_line_below(),
            KeyCode::Char('O') => app.open_new_line_above(),
            KeyCode::Char(':') => {
                app.mode = AppMode::Command;
                app.command_input.clear();
            }
            KeyCode::Char('d') => {
                match app.pending_cmd {
                    Some('d') => {
                        if let KeyCode::Char('d') = key.code {
                            let new_len = app.buffer.delete_line(app.cursor_y);
                            if app.cursor_y >= new_len && app.cursor_y > 0 {
                                app.cursor_y = new_len - 1;
                            }

                            let line_len = app.buffer.lines[app.cursor_y].len();
                            if app.cursor_x > line_len {
                                app.cursor_x = if line_len > 0 { line_len - 1 } else { 0 };
                            }
                            app.pending_cmd = None;
                        } else {
                            app.pending_cmd = None;
                        }
                    }
                    Some(_) => {
                        //TODO 'dw' 'd$'
                        app.pending_cmd = None;
                    }
                    None => {
                        app.pending_cmd = Some('d');
                    }
                }
            }
            KeyCode::Char('D') => {
                app.buffer.kill_line(app.cursor_y, app.cursor_x);
            }
            _ => {}
        }
    }

    fn handle_insert(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Esc => app.mode = AppMode::Normal,
            KeyCode::Enter => app.insert_newline(),
            KeyCode::Backspace => app.handle_backspace(),
            KeyCode::Char(c) => {
                app.buffer.insert_char(app.cursor_y, app.cursor_x, c);
                app.cursor_x += 1;
            }
            _ => {}
        }
    }

    fn handle_command(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
                app.command_input.clear();
            }
            KeyCode::Enter => {
                //app.execute_command();
                let cmd = app.command_input.trim().to_string();
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.is_empty() {
                    app.mode = AppMode::Normal;
                    return;
                }

                match parts[0] {
                    "w" | "write" => {
                        if parts.len() > 1 {
                            app.file_path = Some(PathBuf::from(parts[1]));
                        }
                        let _ = app.save();
                        app.mode = AppMode::Normal;
                    },
                    "q" | "quit" => {
                        app.mode = AppMode::Quit;
                    },
                    "wq" => {
                        let _ = app.save();
                        app.mode = AppMode::Quit;
                    },
                    "e" | "edit" => {
                        if parts.len() > 1 {
                            let _ = app.open(PathBuf::from(parts[1]));
                            app.mode = AppMode::Normal;
                        }
                    },
                    _ => {
                        app.mode = AppMode::Normal;
                    }
                }
                app.command_input.clear();
            }
            KeyCode::Backspace => {
                if app.command_input.is_empty() {
                    app.mode = AppMode::Normal;
                } else {
                    app.command_input.pop();
                }
            }
            KeyCode::Char(c) => {
                app.command_input.push(c);
            }
            _ => {}
        }
    }
}
