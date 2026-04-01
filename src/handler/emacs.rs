use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, AppMode};
use super::KeyHandler;

pub struct EmacsHandler;

impl KeyHandler for EmacsHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) {
        if app.mode == AppMode::Normal {
            app.mode = AppMode::Insert;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            self.handle_control_codes(key.code, app);
        } else if key.modifiers.contains(KeyModifiers::ALT) {
            self.handle_alt_codes(key.code, app);
        } else {
            self.handle_plain_keys(key.code, app);
        }
    }
}

impl EmacsHandler {
    fn handle_control_codes(&self, code: KeyCode, app: &mut App) {
        match code {
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
            KeyCode::Char('g') => app.mode = AppMode::Quit,
            KeyCode::Char('h') => app.handle_backspace(),
            KeyCode::Char('l') => app.center_cursor(),
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
            _ => {}
        }
    }
}
