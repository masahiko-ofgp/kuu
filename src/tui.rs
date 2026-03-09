use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{
        enable_raw_mode,
        EnterAlternateScreen,
    },
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Write, Stdout};


fn set_panic_hook() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let _ = ratatui::restore();

        hook(info);
    }));
}

fn setup_ctrlc() {
    ctrlc::set_handler(move || {
        let _ = ratatui::restore();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
}

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());

        let mut terminal = Terminal::new(backend)?;

        io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        set_panic_hook();
        setup_ctrlc();

        terminal.clear()?;

        Ok(Self { terminal })
    }

    pub fn exit(&mut self) -> Result<()> {
        io::stdout().flush()?;
        ratatui::try_restore()?;
        Ok(())
    }
}
