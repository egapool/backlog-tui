use crate::app::App;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Left Column
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|issue| {
            let summary = vec![Spans::from(vec![
                Span::from(issue.issue_key.clone()),
                Span::from(": "),
                Span::from(issue.summary.clone()),
            ])];
            ListItem::new(summary).style(Style::default())
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
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    // Right Column
    let selected = app.items.state.selected();
    let text = match selected {
        Some(i) => &app.items.items[i].description,
        None => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Issue ", Style::default()));
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);
}
