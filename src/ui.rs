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
    let editor_area = chunks[0];
    
    let inner_height = editor_area.height.saturating_sub(2) as usize;

    let visible_lines: Vec<Line> = app.buffer.lines
        .iter()
        .skip(app.row_offset)
        .take(inner_height)
        .map(|l| Line::from(l.as_str()))
        .collect();

    let editor = Paragraph::new(visible_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" Kuu editor - {:?}", app.mode)));

    f.render_widget(editor, editor_area);

    // 枠線(Borders::ALL)があるので、x+1 y+1のオフセットが必要
    f.set_cursor_position(
        (editor_area.x + 1 + app.cursor_x as u16,
        editor_area.y + 1 + (app.cursor_y - app.row_offset) as u16),
        );


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
