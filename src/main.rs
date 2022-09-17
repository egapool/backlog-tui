use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Issue {
    /// 課題ID
    id: u32,
    /// 課題のキー
    issue_key: String,
    /// タイトル
    summary: String,
    /// 概要
    description: String,
}

struct App {
    items: StatefulList<Issue>,
}

impl App {
    fn with_items(items: Vec<Issue>) -> App {
        App {
            items: StatefulList::with_items(items),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Fetch Contents
    let issues = get_issues().await?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::with_items(issues);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => app.items.unselect(),
                KeyCode::Char('j') => app.items.next(),
                KeyCode::Char('k') => app.items.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
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
                Span::from(" "),
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

async fn get_issues() -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
    let backlog_space_id = env::var("BACKLOG_SPACE_ID")?;
    let backlog_api_key = env::var("BACKLOG_API_KEY")?;

    let url = format!("https://{}.backlog.com/api/v2/issues", backlog_space_id);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(&[
            ("projectId[]", "156939"),
            ("apiKey", &backlog_api_key),
            ("count", "100"),
        ])
        .send()
        .await?;

    // TODO response.status()の中身でエラーハンドリング

    let issues = response.json::<Vec<Issue>>().await?;

    Ok(issues)
}
