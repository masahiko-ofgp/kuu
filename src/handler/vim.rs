use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, AppMode, ConfirmAction, KeyBindMode};
use super::KeyHandler;
use std::path::{PathBuf};


pub struct VimHandler;

impl KeyHandler for VimHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) {
        match app.mode {
            AppMode::Normal => self.handle_normal(key, app),
            AppMode::Insert => {
                if app.is_readonly {
                    app.status_message = Some("Error: File is Read-Only".to_string());
                    app.mode = AppMode::Normal;
                    return;
                } else {
                    self.handle_insert(key, app);
                }
            }
            AppMode::Command => self.handle_command(key, app),
            AppMode::FileTree => self.handle_file_tree(key, app),
            AppMode::Confirm => self.handle_confirm(key, app),
            AppMode::Help => self.handle_help_keys(key, app),
            AppMode::Search => self.handle_search(key, app),
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
            KeyCode::Char('i') => {
                app.history.start_group();
                app.mode = AppMode::Insert;
            },
            KeyCode::Char('x') => app.delete_char(),
            KeyCode::Char('a') => {
                app.cursor_x += 1;
                app.history.start_group();
                app.mode = AppMode::Insert;
            },
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Down => app.move_cursor_down(),
            KeyCode::Up => app.move_cursor_up(),
            KeyCode::Right => app.move_cursor_right(),
            KeyCode::Char('h') => app.move_cursor_left(),
            KeyCode::Char('j') => app.move_cursor_down(),
            KeyCode::Char('k') => app.move_cursor_up(),
            KeyCode::Char('l') => app.move_cursor_right(),
            KeyCode::Char('o') => {
                app.history.start_group();
                app.open_new_line_below();
            },
            KeyCode::Char('O') => {
                app.history.start_group();
                app.open_new_line_above();
            },
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
            KeyCode::Char('/') => {
                app.mode = AppMode::Search;
                app.search_results.clear();
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
                } else {
                    app.undo();
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
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.redo();
                } else {
                    app.pending_cmd = Some('r');
                }
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
                app.history.finish_group();
            }
            KeyCode::Enter => app.insert_newline(),
            KeyCode::Backspace => app.handle_backspace(),
            KeyCode::Char(c) => app.insert_char(c),
            KeyCode::Tab => app.insert_tab(),
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Down => app.move_cursor_down(),
            KeyCode::Up => app.move_cursor_up(),
            KeyCode::Right => app.move_cursor_right(),
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
                    "help" => {
                        app.show_help();
                        app.command_input.clear();
                    },
                    "chkey" => {
                        app.config.key_bind_mode = KeyBindMode::Emacs;
                        app.mode = AppMode::Insert;
                    }
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

    fn handle_help_keys(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Insert;
            }
            KeyCode::Char('j') => {
                app.help_scroll_offset += 1;
            }
            KeyCode::Char('k') => {
                app.help_scroll_offset = app.help_scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn handle_search(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Enter => {
                if !app.search_results.is_empty() {
                    app.current_search_match_idx = (app.current_search_match_idx + 1) % app.search_results.len();
                    app.jump_to_current_search_result();
                    app.status_message = Some(format!("Found {} matches", app.search_results.len()));
                    app.mode = AppMode::Normal;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Normal;
                app.search_results.clear();
                app.status_message = None;
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.execute_search();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.execute_search();
            }
            _ => {}
        }
    }
}
