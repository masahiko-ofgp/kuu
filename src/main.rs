mod app;
mod tui;
mod ui;

use anyhow::Result;
use app::{App, AppMode};
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEventKind,
};
use tui::Tui;


fn main() -> Result<()> {
    let mut app = App::new();
    let mut tui = Tui::new()?;

    while app.mode != AppMode::Quit {
        tui.terminal.draw(|f| ui::render(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }

            match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') => app.mode = AppMode::Quit,
                    KeyCode::Char('i') => app.mode = AppMode::Insert,
                    _ => {}
                },
                AppMode::Insert => match key.code {
                    KeyCode::Esc => app.mode = AppMode::Normal,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    tui.exit()?;
    Ok(())
}
