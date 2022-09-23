use reqwest::{Client, Response};
use std::env;
use std::error::Error;

use crate::app::{Comment, Issue};

pub struct BacklogClient {
    client: Client,
    space_id: String,
    api_key: String,
}

impl BacklogClient {
    pub fn new(space_id: String, api_key: String) -> BacklogClient {
        BacklogClient {
            space_id,
            api_key,
            client: Client::new(),
        }
    }

    async fn get(&self, path: &str, query: &Vec<(&str, &str)>) -> Result<Response, Box<dyn Error>> {
        let url = format!("https://{}.backlog.com/api/v2{}", self.space_id, path);
        let mut q: Vec<(&str, &str)> = vec![("apiKey", &self.api_key)];
        q.extend(query);
        let response = self.client.get(url).query(&q).send().await?;
        Ok(response)
    }

    /// 課題一覧の取得
    pub async fn fetch_issues(&self) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
        let project_id: &str = &env::var("BACKLOG_PROJECT_ID")?;
        let stauts_id_list = env::var("BACKLOG_STATUS_ID_LIST")?;
        let mut query = vec![
            ("projectId[]", project_id),
            ("count", "100"),
            ("sort", "status"),
        ];
        stauts_id_list.split(',').for_each(|id| {
            query.push(("statusId[]", id));
        });

        let response = self.get("/issues", &query).await?;

        // TODO response.status()の中身でエラーハンドリング

        let issues = response.json::<Vec<Issue>>().await?;

        Ok(issues)
    }

    pub async fn fetch_comments(&self, issue: &Issue) -> Result<Vec<Comment>, Box<dyn Error>> {
        let path = format!("/issues/{}/comments", issue.issue_key);
        let query: Vec<(&str, &str)> = vec![("order", "asc"), ("count", "100")];
        let response = self.get(&path, &query).await?;
        let comments = response.json::<Vec<Comment>>().await?;
        let comments = comments
            .iter()
            .filter(|c| c.content != None)
            .cloned()
            .collect();
        Ok(comments)
    }
}
