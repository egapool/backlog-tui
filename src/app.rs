use crate::client::BacklogClient;
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
                    i
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
                    0
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
    pub assignee: Option<User>,
    pub updated: String,
    pub status: Status,
    #[serde(skip_deserializing)]
    pub comments: Option<Vec<Comment>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub content: Option<String>,
    pub created_user: User,
    pub created: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub id: u32,
    pub name: String,
    pub color: String,
}

pub struct App {
    pub items: StatefulList<Issue>,
    base_url: String,
    client: BacklogClient,
    pub is_vertical: bool,
}

impl App {
    pub async fn init() -> Result<App, Box<dyn std::error::Error>> {
        let backlog_space_id = env::var("BACKLOG_SPACE_ID")?;
        let backlog_api_key = env::var("BACKLOG_API_KEY")?;
        let client = BacklogClient::new(backlog_space_id, backlog_api_key);

        let issues = client.fetch_issues().await?;
        let backlog_space_id = env::var("BACKLOG_SPACE_ID")?;
        let app = App {
            items: StatefulList::with_items(issues),
            base_url: format!("https://{}.backlog.com", backlog_space_id),
            client,
            is_vertical: false,
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

    pub async fn on_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // カーソルを1行に下に移動
        self.items.next();

        // 該当issueのコメントを取得
        let issue = match self.items.state.selected() {
            Some(i) => Some(&mut self.items.items[i]),
            None => None,
        };
        if let Some(i) = issue {
            if i.comments.is_none() {
                let comments = self.client.fetch_comments(i).await;
                if let Ok(cs) = comments {
                    i.comments = Some(cs);
                }
            }
        }
        Ok(())
    }
}
