use crossterm::event::{KeyCode, KeyEvent};
use crate::app::{App, AppMode};
use super::KeyHandler;


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
            KeyCode::Char(':') => {
                app.mode = AppMode::Command;
                app.command_input.clear();
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
                app.execute_command();
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
