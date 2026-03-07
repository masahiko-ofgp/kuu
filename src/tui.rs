use anyhow::Result;
use crossterm::{
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::prelude::*;
use std::io::{self, Write, Stdout};
use std::panic;


pub fn install_panic_hook() {
    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        default_hook(info);
    }));
}

pub fn setup_ctrlc() {
    ctrlc::set_handler(move || {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
}

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();

        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);

        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    pub fn exit(&mut self) -> Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)?;
        io::stdout().flush()?;
        disable_raw_mode()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
