use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, AppMode};


pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),  // Editor area
            Constraint::Length(1),  // Status line
        ])
        .split(f.area());


    // ------- Editor area ------
    //
    let editor_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Kuu editor - {:?}", app.mode));

    f.render_widget(editor_block, chunks[0]);


    // ------- Status line ------
    //
    let status_text = match app.mode {
        AppMode::Normal => " -- NORMAL -- ",
        AppMode::Insert => " -- INSERT -- ",
        _ => "",
    };

    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_widget(status_bar, chunks[1]);
}
