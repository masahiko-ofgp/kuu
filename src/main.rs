mod app;
mod tui;
mod ui;
mod buff;

use anyhow::Result;
use app::{App, AppMode};
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEventKind,
};
use tui::Tui;
use std::env;
use::std::path::PathBuf;


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let mut app = if args.len() > 1 {
        App::with_file(PathBuf::from(&args[1]))
    } else {
        App::new()
    };

    let mut tui = Tui::new()?;

    while app.mode != AppMode::Quit {

        let terminal_height = tui.terminal.size()?.height.saturating_sub(3) as usize;
        app.scroll(terminal_height);

        tui.terminal.draw(|f| ui::render(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }

            match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') => app.mode = AppMode::Quit,
                    KeyCode::Char('i') => app.mode = AppMode::Insert,
                    KeyCode::Char('s') => { app.save()?; },
                    KeyCode::Char('h') => app.move_cursor_left(),
                    KeyCode::Char('j') => app.move_cursor_down(),
                    KeyCode::Char('k') => app.move_cursor_up(),
                    KeyCode::Char('l') => app.move_cursor_right(),
                    KeyCode::Char('o') => app.open_new_line_below(),
                    _ => {}
                },
                AppMode::Insert => match key.code {
                    KeyCode::Esc => app.mode = AppMode::Normal,
                    KeyCode::Enter => {
                        app.insert_newline();
                    }
                    KeyCode::Char(c) => {
                        app.buffer.insert_char(app.cursor_y, app.cursor_x, c);
                        app.cursor_x += 1;
                    }
                    KeyCode::Backspace => {
                        app.handle_backspace();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    tui.exit()?;
    Ok(())
}
