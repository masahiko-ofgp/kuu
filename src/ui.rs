use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, AppMode, KeyBindMode};
use unicode_width::UnicodeWidthStr;


pub fn render(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
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
        .borders(Borders::TOP)
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

    let full_text = app.buffer.as_full_text();
    let highlights = app.highlighter.get_highlights(&full_text);
    let mut lines = Vec::new();
    let mut current_byte = 0;

    for line_str in app.buffer.lines.iter() {
        let mut spans = Vec::new();
        let line_start_byte = current_byte;
        let line_end_byte = line_start_byte + line_str.len();

        let mut last_pos = line_start_byte;

        for h in &highlights {
            if h.start_byte < line_end_byte &&
                h.end_byte > line_start_byte {
                    if h.start_byte > last_pos {
                        let start = last_pos - line_start_byte;
                        let end = h.start_byte - line_start_byte;
                        spans.push(Span::raw(&line_str[start..end]));
                    }
                    let h_start = h.start_byte.max(line_start_byte) - line_start_byte;
                    let h_end = h.end_byte.min(line_end_byte) - line_start_byte;
                    spans.push(Span::styled(
                            &line_str[h_start..h_end],
                            Style::default().fg(h.color)
                            ));
                    last_pos = h.end_byte.min(line_end_byte);
                }
        }
        if last_pos < line_end_byte {
            spans.push(Span::raw(&line_str[last_pos - line_start_byte..]));
        }
        lines.push(Line::from(spans));
        current_byte = line_end_byte + 1;
    }
    let display_lines: Vec<Line> = lines
        .into_iter()
        .skip(app.row_offset)
        .take(inner_editor_area.height as usize)
        .collect();

    let editor_widget = Paragraph::new(display_lines)
        .block(editor_block)
        .wrap( Wrap { trim: false });

    f.render_widget(editor_widget, body_area);

    if let Some(current_line) = app.buffer.lines.get(app.cursor_y) {
        let cursor_x_display = current_line
            .chars()
            .take(app.cursor_x)
            .collect::<String>()
            .width();
        f.set_cursor_position(
            (inner_editor_area.x + cursor_x_display as u16,
             inner_editor_area.y + (app.cursor_y - app.row_offset) as u16)
            );
    }

    // ------- Status line ------
    //
    let lang_name = app.highlighter.current_language_name()
        .unwrap_or("Plain Text");

    let vim_status_text = format!("[{:?}]|ROW:{}COL:{}|FILE: {}|{}|{}",
        app.mode,
        app.cursor_y + 1,
        app.cursor_x,
        app.file_path.as_ref().map(|p| p.to_str()
            .unwrap_or("NO NAME")).unwrap_or("NO NAME"),
        lang_name,
        app.show_status_message(),
        );
    let other_status_text = format!("ROW:{}COL:{}|FILE:{}|{}|{}",
        app.cursor_y + 1,
        app.cursor_x,
        app.file_path.as_ref().map(|p| p.to_str()
            .unwrap_or("NO NAME")).unwrap_or("NO NAME"),
        lang_name,
        app.show_status_message(),
        );

    if app.config.key_bind_mode == KeyBindMode::Vim {
        let status_bar = Paragraph::new(vim_status_text)
            .style(Style::default().bg(Color::Cyan).fg(Color::White));

        f.render_widget(status_bar, main_chunks[1]);
    } else {
        let status_bar = Paragraph::new(other_status_text)
            .style(Style::default().bg(Color::White).fg(Color::Cyan));

        f.render_widget(status_bar, main_chunks[1]);
    }

    // Command line
    //
    match app.mode {
        AppMode::Command => {
            let cmd_text = format!(":{}", app.command_input);
            let cmd_display_width = UnicodeWidthStr::width(app.command_input.as_str());
            f.render_widget(
                Paragraph::new(cmd_text).style(
                    Style::default().fg(Color::Yellow)),
                main_chunks[2]
            );

            f.set_cursor_position(
                (main_chunks[2].x + (cmd_display_width + 1) as u16,
                 main_chunks[2].y)
                );
        }
        _ => {
            f.render_widget(Paragraph::new(""), main_chunks[2]);
        }
    }


}
