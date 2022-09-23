use crate::app::App;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let dir = if app.is_vertical {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };
    let chunks = Layout::default()
        .direction(dir)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(f.size());

    // Left Column
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|issue| {
            let status_color = hex_to_rgb_color(&issue.status.color);
            let mut lines: Vec<Spans> = vec![];
            let mut line = vec![
                Span::styled(issue.issue_key.clone(), Style::default().fg(status_color)),
                Span::styled(" ", Style::default()),
                Span::styled(issue.status.name.clone(), Style::default().fg(status_color)),
                Span::styled(" ", Style::default()),
                Span::styled(issue.summary.clone(), Style::default().fg(status_color)),
            ];
            match &issue.assignee {
                Some(i) => line.push(Span::styled(
                    format!(" @{}", i.name.clone()),
                    Style::default().fg(status_color),
                )),
                None => {}
            }
            line.push(Span::styled(" ", Style::default()));
            line.push(Span::styled(
                issue.updated.clone(),
                Style::default().fg(status_color),
            ));
            lines.push(Spans::from(line));
            ListItem::new(lines).style(Style::default())
        })
        .collect();

    let items = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" List of Issue"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 50))
                // .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    // Right Column
    let mut text = String::from("");

    if let Some(i) = app.get_selected_issue() {
        text = i.description.clone();

        if let Some(cs) = &i.comments {
            for c in cs.iter() {
                if let Some(com) = &c.content {
                    let t = format!("\n\n💬 {} {}:  {}", c.created_user.name, c.created, com);
                    text = text + &t;
                }
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Issue ", Style::default()));
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Rgb(200, 200, 200)))
        .alignment(Alignment::Left);
    // .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);
}

fn hex_to_rgb_color(hex: &str) -> Color {
    let rgb = (1..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>();
    Color::Rgb(rgb[0], rgb[1], rgb[2])
}
