mod app;
mod tui;
mod ui;
mod buff;
mod handler;
mod config;
mod highlight;

use anyhow::Result;
use app::{App, AppMode};
use config::Config;
use crossterm::event::{
    self,
    Event,
    KeyEventKind,
};
use tui::Tui;
use std::env;
use std::path::PathBuf;


fn main() -> Result<()> {
    let config = Config::load();

    let args: Vec<String> = env::args().collect();
    
    let mut app = if args.len() > 1 {
        App::with_file(PathBuf::from(&args[1]), config)
    } else {
        App::with_config(config)
    };

    let mut tui = Tui::new()?;

    while app.mode != AppMode::Quit {

        let terminal_height = tui.terminal.size()?.height.saturating_sub(3) as usize;
        app.scroll(terminal_height);

        tui.terminal.draw(|f| ui::render(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }

            let handler = handler::get_handler(app.config.key_bind_mode);
            handler.handle_key(key, &mut app);
        }
    }

    tui.exit()?;
    Ok(())
}
