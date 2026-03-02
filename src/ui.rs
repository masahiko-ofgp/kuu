use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::App;
use crate::app::KeyBindMode;


pub fn render(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());


    // ------- Editor area ------
    //
    let line_num_width = if app.config.show_line_numbers {4} else {0};
    
    let editor_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(line_num_width),
            Constraint::Min(0),
        ])
        .split(main_chunks[0]);

    let line_num_area = editor_chunks[0];
    let body_area = editor_chunks[1];

    let editor_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Kuu "));

    let inner_editor_area = editor_block.inner(body_area);

    if app.config.show_line_numbers {
        let mut lines = Vec::new();

        for i in 0..line_num_area.height {
            let line_idx = app.row_offset + i as usize;

            if line_idx < app.buffer.lines.len() {
                lines.push(Line::from(format!("{:3} \n", line_idx + 1)));
            } else {
                lines.push(Line::from("~  "));
            }
        }

        let line_num_widget = Paragraph::new(lines)
            .style(Style::default().fg(Color::DarkGray).bg(Color::Reset));

        let mut line_num_rect = line_num_area;
        line_num_rect.y = inner_editor_area.y;
        line_num_rect.height = inner_editor_area.height;

        f.render_widget(line_num_widget, line_num_rect);
    }

    let visible_lines: Vec<Line> = app.buffer.lines
        .iter()
        .skip(app.row_offset)
        .take(inner_editor_area.height as usize)
        .map(|l| Line::from(l.as_str()))
        .collect();

    let editor_widget = Paragraph::new(visible_lines)
        .block(editor_block);

    f.render_widget(editor_widget, body_area);

    f.set_cursor_position(
        (inner_editor_area.x + app.cursor_x as u16,
        inner_editor_area.y + (app.cursor_y - app.row_offset) as u16),
        );


    // ------- Status line ------
    //
    let vim_status_text = format!(" [{:?}] | ROW: {}  COL: {} | FILE: {} ",
        app.mode,
        app.cursor_y + 1,
        app.cursor_x,
        app.file_path.as_ref().map(|p| p.to_str()
            .unwrap_or("NO NAME")).unwrap_or("NO NAME")
        );
    let other_status_text = format!(" ROW: {}  COL: {} | FILE: {}",
        app.cursor_y + 1,
        app.cursor_x,
        app.file_path.as_ref().map(|p| p.to_str()
            .unwrap_or("NO NAME")).unwrap_or("NO NAME")
        );

    if app.config.key_bind_mode == KeyBindMode::Vim {
        let status_bar = Paragraph::new(vim_status_text)
            .style(Style::default().bg(Color::Blue).fg(Color::White));

        f.render_widget(status_bar, main_chunks[1]);
    } else {
        let status_bar = Paragraph::new(other_status_text)
            .style(Style::default().bg(Color::White).fg(Color::Blue));

        f.render_widget(status_bar, main_chunks[1]);
    }
}
