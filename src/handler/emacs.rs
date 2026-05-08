use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, AppMode, ConfirmAction, KeyBindMode};
use super::KeyHandler;
use std::path::PathBuf;


pub struct EmacsHandler;

impl KeyHandler for EmacsHandler {
    fn handle_key(&self, key: KeyEvent, app: &mut App) {
        if app.mode == AppMode::Help {
            self.handle_help_keys(key, app);
            return;
        }

        if app.mode == AppMode::FileTree {
            self.handle_file_tree(key, app);
            return;
        }

        if app.mode == AppMode::Command {
            self.handle_command(key, app);
            return;
        }

        if app.mode == AppMode::Confirm {
            self.handle_confirm(key, app);
            return;
        }

        if app.mode == AppMode::Search {
            self.handle_search_keys(key, app);
            return;
        }

        if app.mode == AppMode::Normal {
            if app.is_readonly {
                app.mode = AppMode::Normal;
                app.status_message = Some("File is Read-Only".to_string());
                return;
            } else {
                app.mode = AppMode::Insert;
            }
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
            KeyCode::Char('d') => app.delete_char(),
            KeyCode::Char('k') => {
                app.kill_line();
                app.history.finish_group();
            }
            KeyCode::Char('o') => {
                let saved_x = app.cursor_x;
                let saved_y = app.cursor_y;
                app.insert_newline();
                app.cursor_x = saved_x;
                app.cursor_y = saved_y;
            }
            KeyCode::Char('y') => {
                app.put_before();
            }
            KeyCode::Char('g') => {
                if app.is_buffer_modified() {
                    app.request_confirm("Discord unsaved changes?", ConfirmAction::Quit);
                } else {
                    app.mode = AppMode::Quit
                }
            }
            KeyCode::Char('h') => app.show_help(),
            KeyCode::Char('l') => app.center_cursor(),
            KeyCode::Char('v') => app.scroll_half_page_down(),
            KeyCode::Char('/') => app.undo(),
            KeyCode::Char('_') if key.modifiers.contains(KeyModifiers::SHIFT) => app.redo(),
            KeyCode::Char('s') => {
                app.mode = AppMode::Search;
                app.search.clear();
                app.status_message = Some("Search: ".to_string());
            }
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
                app.cursor_x = app.buffer.lines[app.cursor_y].chars().count();
            }
            KeyCode::Char('v') => app.scroll_half_page_up(),
            KeyCode::Char('x') => {
                app.mode = AppMode::Command;
                app.command_input.clear();
                app.status_message = None;
            }
            _ => {}
        }
    }

    fn handle_plain_keys(&self, code: KeyCode, app: &mut App) {
        match code {
            KeyCode::Enter => app.insert_newline(),
            KeyCode::Backspace => app.handle_backspace(),
            KeyCode::Char(c) => app.insert_char(c),
            KeyCode::Esc => {
                app.pending_cmd = None;
                app.status_message = Some("Quit".to_string());
            }
            KeyCode::Tab => app.insert_tab(),
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Down => app.move_cursor_down(),
            KeyCode::Up => app.move_cursor_up(),
            KeyCode::Right => app.move_cursor_right(),
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
            ('x', KeyCode::Char('f')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.mode = AppMode::Command;
                app.command_input = "find-file ".to_string();
            }
            ('x', KeyCode::Char('u')) => app.undo(),
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

    fn handle_command(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Enter => {
                let input = app.command_input.trim().to_string();
                let parts: Vec<&str> = input.split_whitespace().collect();
                if !parts.is_empty() {
                    match parts[0] {
                        "quit" => {
                            if app.is_buffer_modified() {
                                app.request_confirm(
                                    "Discord unsaved changes?",
                                    ConfirmAction::Quit
                                    );
                            } else {
                                app.mode = AppMode::Quit;
                            }
                        }
                        "kill-buffer" => app.close_file(),
                        "find-file" => {
                            if parts.len() > 1 {
                                app.open(PathBuf::from(parts[1]));
                            }
                        }
                        "save-buffer" => app.save_and_reload(),
                        "chkey" => {
                            app.config.key_bind_mode = KeyBindMode::Vim;
                        }
                        "help" => {
                            app.show_help();
                            app.command_input.clear();
                        }
                        _ => {
                            app.status_message = Some(format!("Unknown command: {}", parts[0]));
                        }
                    }
                }
                app.command_input.clear();
                if app.mode == AppMode::Command {
                    app.mode = AppMode::Insert;
                }
            }
            KeyCode::Esc | KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.command_input.clear();
                app.mode = AppMode::Insert;
            }
            KeyCode::Char(c) => {
                app.command_input.push(c);
            }
            KeyCode::Backspace => {
                app.command_input.pop();
            }
            _ => {}
        }
    }
    
    fn handle_help_keys(&self, key: KeyEvent, app: &mut App) {
        let help_content = app.get_help_content();
        let total_lines = help_content.len();
        let visible_height = ((app.file_viewport_height as f32 / 0.8) * 0.8) as usize;
        let max_offset = total_lines.saturating_sub(visible_height);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Insert;
            }
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.mode = AppMode::Insert;
            }
            KeyCode::Char('n') | KeyCode::Down => {
                if app.help_scroll_offset < max_offset {
                    app.help_scroll_offset += 1;
                }
            }
            KeyCode::Char('p') | KeyCode::Up => {
                app.help_scroll_offset = app.help_scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn handle_search_keys(&self, key: KeyEvent, app: &mut App) {
        match key.code {
            KeyCode::Enter => {
                app.mode = AppMode::Insert;
                app.status_message = Some(format!("Found {} matches", app.search.results.len()));
            }
            KeyCode::Esc | KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.mode = AppMode::Insert;
                app.search.clear();
                app.status_message = None;
            }
            KeyCode::Backspace => {
                app.search.query.pop();
                app.execute_search();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !app.search.results.is_empty() {
                    app.search.current_match_idx = (app.search.current_match_idx + 1) % app.search.results.len();
                    app.jump_to_current_search_result();
                }
            }
            KeyCode::Char(c) => {
                app.search.query.push(c);
                app.execute_search();
            }
            _ => {}
        }
    }
}
