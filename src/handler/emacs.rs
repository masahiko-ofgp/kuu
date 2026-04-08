use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, AppMode, ConfirmAction};
use super::KeyHandler;

pub struct EmacsHandler;

impl KeyHandler for EmacsHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) {
        if app.mode == AppMode::Normal {
            app.mode = AppMode::Insert;
        }

        if app.mode == AppMode::FileTree {
            self.handle_file_tree(key, app);
            return;
        }

        if app.mode == AppMode::Confirm {
            self.handle_confirm(key, app);
            return;
        }

        if let Some(prefix) = app.pending_cmd {
            self.handle_prefix_codes(prefix, key, app);
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            self.handle_control_codes(key, app);
        } else if key.modifiers.contains(KeyModifiers::ALT) {
            self.handle_alt_codes(key.code, app);
        } else {
            self.handle_plain_keys(key.code, app);
        }
    }
}

impl EmacsHandler {
    fn handle_control_codes(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('x') => app.pending_cmd = Some('x'),
            KeyCode::Char('p') => app.move_cursor_up(),
            KeyCode::Char('n') => app.move_cursor_down(),
            KeyCode::Char('b') => app.move_cursor_left(),
            KeyCode::Char('f') => app.move_cursor_right(),
            KeyCode::Char('a') => app.cursor_x = 0,
            KeyCode::Char('e') => {
                app.cursor_x = app.buffer.lines[app.cursor_y].chars().count();
            }
            KeyCode::Char('d') => {
                app.buffer.delete_char(app.cursor_y, app.cursor_x);
            }
            KeyCode::Char('k') => {
                app.kill_line();
            }
            KeyCode::Char('y') => {
                app.put_before();
            }
            KeyCode::Char('g') => {
                if app.is_buffer_modified() {
                    app.status_message = Some("File modified! Save or discord first.".to_string());
                } else {
                    app.mode = AppMode::Quit
                }
            }
            KeyCode::Char('h') => app.handle_backspace(),
            KeyCode::Char('l') => app.center_cursor(),
            KeyCode::Char('v') => app.scroll_half_page_down(),
            _ => {}
        }
    }

    fn handle_alt_codes(&self, code: KeyCode, app: &mut App) {
        match code {
            KeyCode::Char('f') => app.move_word_forward(),
            KeyCode::Char('b') => app.move_word_backward(),
            KeyCode::Char('<') => {
                app.cursor_y = 0;
                app.cursor_x = 0;
            }
            KeyCode::Char('>') => {
                app.cursor_y = app.buffer.lines.len()
                    .saturating_sub(1);
                app.cursor_x = 0;
                app.snap_cursor();
            }
            KeyCode::Char('v') => app.scroll_half_page_up(),
            _ => {}
        }
    }

    fn handle_plain_keys(&self, code: KeyCode, app: &mut App) {
        match code {
            KeyCode::Enter => app.insert_newline(),
            KeyCode::Backspace => app.handle_backspace(),
            KeyCode::Char(c) => {
                app.buffer.insert_char(app.cursor_y, app.cursor_x, c);
                app.cursor_x += 1;
            }
            KeyCode::Esc => {
                app.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_prefix_codes(&self, prefix: char, key: KeyEvent, app: &mut App)
    {
        app.clear_pending();

        match (prefix, key.code) {
            ('x', KeyCode::Char('d')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.mode = AppMode::FileTree;
            }
            ('x',KeyCode::Char('s')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.save_and_reload();
            }
            ('x', KeyCode::Char('c')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if app.is_buffer_modified() {
                    app.request_confirm(
                        "Discord unsaved changes?",
                        ConfirmAction::Quit
                        );
                } else {
                    app.mode = AppMode::Quit;
                }
            }
            ('x', KeyCode::Char('k')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.close_file();
            }
            ('x', KeyCode::Char('e')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.open_config();
            }
            _ => {
                app.status_message = Some(format!("C-{} {} is undefined", prefix, key.code));
            }
        }
    }

    fn handle_file_tree(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('j') => app.file_tree_next(),
            KeyCode::Char('k') => app.file_tree_prev(),
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => app.file_tree_next(),
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => app.file_tree_prev(),
            KeyCode::Enter | KeyCode::Char('f') => app.file_tree_select(),
            KeyCode::Char('^') => app.file_tree_parent(),
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.mode = AppMode::Insert;
            }
            _ => {}
        }
    }

    fn handle_confirm(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_action(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.cancel_confirm();
            }
            _ => {}
        }
    }
}
