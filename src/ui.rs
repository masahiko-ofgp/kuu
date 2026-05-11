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
        ].as_ref())
        .split(f.area());

    app.update_viewport_height(main_chunks[0].height);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(if app.tree.show { 20 } else { 0 }),
            Constraint::Min(0),
        ])
        .split(main_chunks[0]);

    let tree_area = content_chunks[0];
    let editor_area = content_chunks[1];

    // --- File tree ---
    if app.tree.show {
        app.file_viewport_height = tree_area.height.saturating_sub(2);
        let items: Vec<ListItem> = app.tree.list
            .iter()
            .enumerate()
            .skip(app.tree.offset)
            .take(app.file_viewport_height as usize)
            .map(|(i, path)| {
                let is_selected = i == app.tree.selected;

                let icon = if path.is_dir() {
                    "\u{f4d3}"
                } else {
                    "\u{f15b}"
                };

                let name = if let Some(parent) = app.tree.current_dir.parent() {
                    if path== parent {
                        "..".to_string()
                    } else {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?")
                            .to_string()
                    }
                } else {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string()
                };

                let (base_style, icon_style) = if is_selected {
                    (
                        Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD),
                        Style::default().bg(Color::Blue).fg(Color::Yellow),
                    )
                } else {
                    (
                        Style::default().fg(Color::Gray),
                        Style::default().fg(if path.is_dir() { Color::Cyan} else {Color::White})
                    )
                };

                let line = Line::from(vec![
                    Span::styled(format!(" {} ", icon), icon_style),
                    Span::styled(name, base_style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let tree_block = Block::default()
            .borders(Borders::ALL)
            .title(" Explorer ")
            .border_style(if app.mode == AppMode::FileTree {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let tree_list = List::new(items).block(tree_block);
        f.render_widget(tree_list, tree_area);

    }

    // --- Editor area ---

    let line_num_width = if app.config.show_line_numbers {4} else {0};
    
    let editor_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(line_num_width),
            Constraint::Min(0),
        ])
        .split(editor_area);

    let line_num_area = editor_chunks[0];
    let body_area = editor_chunks[1];

    let editor_block = Block::default()
        .borders(Borders::TOP)
        .title(format!(" Kuu "));

    let inner_editor_area = editor_block.inner(body_area);

    app.scroll(inner_editor_area.height as usize);

    if app.config.show_line_numbers {
        let mut lines = Vec::new();

        for i in 0..line_num_area.height {
            let line_idx = app.view.row_offset + i as usize;

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
    let mut all_lines = Vec::new();
    let mut current_byte = 0;

    for (y_idx, line_str) in app.buffer.lines.iter().enumerate() {
        let line_start_byte = current_byte;
        let line_end_byte = line_start_byte + line_str.len();

        let has_search_hit = !app.search.query.is_empty() && line_str.contains(&app.search.query);

        let spans = if has_search_hit {
            render_search_line_spans(line_str, app, y_idx)
        } else {
            let mut line_spans = Vec::new();
            let mut chars = line_str.char_indices().peekable();
            let relevant_highlights = highlights.iter()
                .filter(|h| h.start_byte < line_end_byte && h.end_byte > line_start_byte);

            for h in relevant_highlights {
                let h_start_rel = h.start_byte.saturating_sub(line_start_byte);
                let h_end_rel = h.end_byte.saturating_sub(line_start_byte);
                let mut normal_text = String::new();

                while let Some(&(byte_pos, c)) = chars.peek() {
                    if byte_pos < h_start_rel {
                        normal_text.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !normal_text.is_empty() {
                    line_spans.push(Span::raw(normal_text));
                }

                let mut highlighted_text = String::new();

                while let Some(&(byte_pos, c)) = chars.peek() {
                    if byte_pos < h_end_rel {
                        highlighted_text.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !highlighted_text.is_empty() {
                    line_spans.push(Span::styled(highlighted_text, Style::default().fg(h.color)));
                }
            }
            let remaining_text: String = chars.map(|(_, c)| c).collect();
            if !remaining_text.is_empty() {
                line_spans.push(Span::raw(remaining_text));
            }

            if line_spans.is_empty() && line_str.is_empty() {
                line_spans.push(Span::raw(""));
            }
            line_spans
        };
        all_lines.push(Line::from(spans));
        current_byte = line_end_byte + 1;
    }
    
    let display_lines: Vec<Line> = all_lines
        .into_iter()
        .skip(app.view.row_offset)
        .take(inner_editor_area.height as usize)
        .collect();

    let editor_widget = Paragraph::new(display_lines)
        .block(editor_block)
        .wrap( Wrap { trim: false });

    f.render_widget(editor_widget, body_area);


    // ------- Status line ------
    //
    let lang_name = app.highlighter.current_language_name()
        .unwrap_or("Plain Text");

    let vim_status_text = format!("[{:?}]|ROW:{}COL:{}|FILE: {}|{}",
        app.mode,
        app.view.cursor_y + 1,
        app.view.cursor_x,
        app.file_path.as_ref().map(|p| p.to_str()
            .unwrap_or("NO NAME")).unwrap_or("NO NAME"),
        lang_name,
        );
    let other_status_text = format!("ROW:{}COL:{}|FILE:{}|{}",
        app.view.cursor_y + 1,
        app.view.cursor_x,
        app.file_path.as_ref().map(|p| p.to_str()
            .unwrap_or("NO NAME")).unwrap_or("NO NAME"),
        lang_name,
        );

    if app.config.key_bind_mode == KeyBindMode::Vim {
        let status_bar = Paragraph::new(vim_status_text)
            .style(Style::default()
                .bg(if app.is_buffer_modified() { Color::White } else { Color::Indexed(046) })
                .fg(if app.is_buffer_modified() { Color::Red } else { Color::Black })
                .add_modifier(Modifier::BOLD));

        f.render_widget(status_bar, main_chunks[1]);
    } else {
        let status_bar = Paragraph::new(other_status_text)
            .style(Style::default()
                .bg(if app.is_buffer_modified() { Color::White } else { Color::Indexed(069) })
                .fg(if app.is_buffer_modified() { Color::Red } else { Color::White })
                .add_modifier(Modifier::BOLD));

        f.render_widget(status_bar, main_chunks[1]);
    }

    if app.mode != AppMode::FileTree {
        if let Some(line) = app.buffer.lines.get(app.view.cursor_y) {
            let cx = line.chars()
                .take(app.view.cursor_x)
                .collect::<String>()
                .width();

            f.set_cursor_position(Position {
                x: inner_editor_area.x + cx as u16,
                y: inner_editor_area.y + (app.view.cursor_y - app.view.row_offset) as u16,
            });
        }
    }

    match app.mode {
        AppMode::Command => {
            let cmd_text = format!(":{}", app.command_input);
            let cmd_display_width = UnicodeWidthStr::width(app.command_input.as_str());
            f.render_widget(
                Paragraph::new(cmd_text).style(
                    Style::default().fg(Color::Yellow)),
                main_chunks[2]
            );

            f.set_cursor_position(Position {
                x: main_chunks[2].x + (cmd_display_width + 1) as u16,
                y: main_chunks[2].y,
            });
        }
        AppMode::Confirm => {
            let confirm_text = app.status_message.as_deref().unwrap_or("Confirm? (y/n)");
            let confirm_display_width = UnicodeWidthStr::width(confirm_text);
            f.render_widget(
                Paragraph::new(confirm_text).style(
                    Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)),
                main_chunks[2]
                );
            f.set_cursor_position(Position {
                x: main_chunks[2].x + (confirm_display_width + 1) as u16,
                y: main_chunks[2].y,
            });
        }
        AppMode::Help => {
            render_help_popup(f, app);
        }
        AppMode::Search => {
            let search_prompt = format!("I-search: {}", app.search.query);
            let search_width = UnicodeWidthStr::width(search_prompt.as_str());
            f.render_widget(
                Paragraph::new(search_prompt)
                .style(Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)),
                main_chunks[2]
                );
            f.set_cursor_position(Position {
                x: main_chunks[2].x + search_width as u16,
                y: main_chunks[2].y,
            });
        }
        _ => {
            if let Some(ref msg) = app.status_message {
                f.render_widget(Paragraph::new(msg.as_str()), main_chunks[2]);
            }

            if let Some(current_line) = app.buffer.lines.get(app.view.cursor_y) {
                let prefix: String = current_line.chars()
                    .take(app.view.cursor_x)
                    .collect();
                let cursor_x_display = UnicodeWidthStr::width(prefix.as_str());
                f.set_cursor_position(Position {
                    x: inner_editor_area.x + cursor_x_display as u16,
                    y: inner_editor_area.y + (app.view.cursor_y - app.view.row_offset) as u16,
                });
            }
        }
    }


}

fn render_help_popup(f: &mut Frame, app: &App) {
    let area = f.area();

    let block = Block::default()
        .title(" Help (Press 'q'or 'Esc' to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .bg(Color::Black);

    let popup_area = centered_rect(80, 80, area);

    f.render_widget(Clear, popup_area);

    let help_lines: Vec<Line> = app.get_help_content()
        .into_iter()
        .map(|l| Line::from(Span::raw(l)))
        .collect();

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll_offset as u16, 0));

    f.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_search_line_spans(line_str: &str, app: &App, y_idx: usize) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = line_str.char_indices().peekable();

    for (match_start_byte, matched_str) in line_str.match_indices(&app.search.query) {
        let match_end_byte = match_start_byte + matched_str.len();
        let mut normal_text = String::new();

        while let Some(&(byte_pos, c)) = chars.peek() {
            if byte_pos < match_start_byte {
                normal_text.push(c);
                chars.next();
            } else {
                break;
            }
        }
        if !normal_text.is_empty() {
            spans.push(Span::raw(normal_text));
        }

        let char_idx = line_str[..match_start_byte].chars().count();
        let is_current = app.search.results.get(app.search.current_match_idx).map_or(false, |m| m.line_idx == y_idx && m.char_idx == char_idx);

        let style = if is_current {
            Style::default().bg(Color::Rgb(255, 165, 0)).fg(Color::Black)
        } else {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        };

        let mut match_text = String::new();

        while let Some(&(byte_pos, c)) = chars.peek() {
            if byte_pos < match_end_byte {
                match_text.push(c);
                chars.next();
            } else {
                break;
            }
        }
        spans.push(Span::styled(match_text, style));
    }
    let remaining: String = chars.map(|(_, c)| c).collect();
    
    if !remaining.is_empty() {
        spans.push(Span::raw(remaining));
    }

    spans
}
