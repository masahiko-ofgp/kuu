use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, AppMode, ConfirmAction};
use super::KeyHandler;
use std::path::{PathBuf};


pub struct VimHandler;

impl KeyHandler for VimHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) {
        match app.mode {
            AppMode::Normal => self.handle_normal(key, app),
            AppMode::Insert => self.handle_insert(key, app),
            AppMode::Command => self.handle_command(key, app),
            AppMode::FileTree => self.handle_file_tree(key, app),
            AppMode::Confirm => self.handle_confirm(key, app),
            _ => {}
        }
    }
}

impl VimHandler {
    fn handle_normal(&self, key: KeyEvent, app: &mut App) {
        if let Some(op) = app.pending_cmd {
            match (op, key.code) {
                ('d', KeyCode::Char('d')) => app.delete_current_line(),
                ('g', KeyCode::Char('g')) => {
                    app.cursor_y = 0;
                    app.cursor_x = 0;
                }
                ('y', KeyCode::Char('y')) => app.yank_current_line(),
                ('>', KeyCode::Char('>')) => app.indent_current_line(),
                ('<', KeyCode::Char('<')) => app.unindent_current_line(),
                ('c', KeyCode::Char('c')) => app.change_current_line(),
                ('r', KeyCode::Char(c)) => app.replace_char(c),
                ('z', KeyCode::Char('z')) => app.center_cursor(),
                _ => {}
            }
            app.clear_pending();
            return;
        }
        match key.code {
            KeyCode::Char('i') => app.mode = AppMode::Insert,
            KeyCode::Char('x') => app.buffer.delete_char(app.cursor_y, app.cursor_x),
            KeyCode::Char('h') => app.move_cursor_left(),
            KeyCode::Char('j') => app.move_cursor_down(),
            KeyCode::Char('k') => app.move_cursor_up(),
            KeyCode::Char('l') => app.move_cursor_right(),
            KeyCode::Char('o') => app.open_new_line_below(),
            KeyCode::Char('O') => app.open_new_line_above(),
            KeyCode::Char('w') => app.move_word_forward(),
            KeyCode::Char('b') => app.move_word_backward(),
            KeyCode::Char('0') => app.cursor_x = 0,
            KeyCode::Char('$') => {
                app.cursor_x = app.buffer.lines[app.cursor_y]
                    .chars()
                    .count()
                    .saturating_sub(1);
            }
            KeyCode::Char(':') => {
                app.mode = AppMode::Command;
                app.command_input.clear();
            }
            KeyCode::Char('t') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.mode = AppMode::FileTree;
                }
            }
            KeyCode::Char('D') => {
                app.kill_line();
            }
            KeyCode::Char('C') => {
                app.change_to_end_of_line();
            }
            KeyCode::Char('p') => {
                app.put_after();
                app.clear_pending();
            }
            KeyCode::Char('P') => {
                app.put_before();
                app.clear_pending();
            }
            KeyCode::Char('d') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.scroll_half_page_down();
                } else {
                    app.pending_cmd = Some('d');
                }
            }
            KeyCode::Char('u') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.scroll_half_page_up();
                }
            }
            KeyCode::Char('g') => {
                app.pending_cmd = Some('g');
            }
            KeyCode::Char('y') => {
                app.pending_cmd = Some('y');
            }
            KeyCode::Char('c') => {
                app.pending_cmd = Some('c');
            }
            KeyCode::Char('>') => {
                app.pending_cmd = Some('>');
            }
            KeyCode::Char('<') => {
                app.pending_cmd = Some('<');
            }
            KeyCode::Char('r') => {
                app.pending_cmd = Some('r');
            }
            KeyCode::Char('z') => {
                app.pending_cmd = Some('z');
            }
            _ => {}
        }
    }

    fn handle_insert(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Esc => {
                app.move_cursor_left();
                app.mode = AppMode::Normal;
            }
            KeyCode::Enter => app.insert_newline(),
            KeyCode::Backspace => app.handle_backspace(),
            KeyCode::Char(c) => {
                app.buffer.insert_char(app.cursor_y, app.cursor_x, c);
                app.cursor_x += 1;
            }
            KeyCode::Tab => {
                app.insert_tab();
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
                        let _ = app.save_and_reload();
                        app.mode = AppMode::Normal;
                    },
                    "q" | "quit" => {
                        if app.is_buffer_modified() {
                            app.request_confirm("Discord unsaved changes?", ConfirmAction::Quit);
                        } else {
                            app.mode = AppMode::Quit;
                        }

                        /*if app.file_path.is_some() || !app.buffer.lines.is_empty() && app.buffer.lines != vec![""] {
                            app.close_file();
                            app.mode = AppMode::Normal;
                        } else {
                            app.mode = AppMode::Quit;
                        }*/
                    },
                    "q!" => {
                        app.mode = AppMode::Quit;
                    }
                    "wq" => {
                        let _ = app.save_and_reload();
                        app.mode = AppMode::Quit;
                    },
                    "e" | "edit" => {
                        if parts.len() > 1 {
                            let _ = app.open(PathBuf::from(parts[1]));
                            app.mode = AppMode::Normal;
                        }
                    },
                    "close" => app.close_file(),
                    "t" | "tree" => {
                        app.show_file_tree = !app.show_file_tree;

                        if app.show_file_tree {
                            app.mode = AppMode::FileTree;
                        } else {
                            app.mode = AppMode::Normal;
                        }
                    }
                    "config" => app.open_config(),
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
    fn handle_file_tree(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Normal;
            }
            KeyCode::Char('j') => app.file_tree_next(),
            KeyCode::Char('k') => app.file_tree_prev(),
            KeyCode::Enter => app.file_tree_select(),
            KeyCode::Backspace | KeyCode::Char('h') => app.file_tree_parent(),
            _ => {}
        }
    }

    fn handle_confirm(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.confirm_action();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.cancel_confirm();
            }
            _ => {}
        }
    }
}
