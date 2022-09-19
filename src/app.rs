use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: u32,
    pub issue_key: String,
    pub summary: String,
    pub description: String,
    pub assignee: Option<Assignee>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Assignee {
    pub name: String,
}

pub struct App {
    pub items: StatefulList<Issue>,
    base_url: String,
}

impl App {
    pub async fn init() -> Result<App, Box<dyn std::error::Error>> {
        let issues = fetch_issues().await?;
        let backlog_space_id = env::var("BACKLOG_SPACE_ID")?;
        let app = App {
            items: StatefulList::with_items(issues),
            base_url: format!("https://{}.backlog.com", backlog_space_id),
        };
        Ok(app)
    }

    pub fn get_selected_issue(&self) -> Option<&Issue> {
        match self.items.state.selected() {
            Some(i) => Some(&self.items.items[i]),
            None => None,
        }
    }

    pub fn open_browser(&self) {
        let selected = self.get_selected_issue();
        match selected {
            Some(i) => {
                let url = format!("{}/view/{}", self.base_url, i.issue_key);
                if Command::new("sh")
                    .arg("-c")
                    .arg(format!("open {}", url))
                    .output()
                    .is_ok()
                {}
            }
            None => (),
        };
    }
}

pub async fn fetch_issues() -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
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
