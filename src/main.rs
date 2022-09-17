use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let issues = get_issues().await?;
    for issue in issues.iter() {
        println!("{}: {}", issue.issue_key, issue.summary);
    }
    Ok(())
}

async fn get_issues() -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
    let backlog_space_id = env::var("BACKLOG_SPACE_ID")?;
    let backlog_api_key = env::var("BACKLOG_API_KEY")?;

    let url = format!("https://{}.backlog.com/api/v2/issues", backlog_space_id);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(&[("projectId[]", "156939"), ("apiKey", &backlog_api_key)])
        .send()
        .await?;

    // TODO response.status()の中身でエラーハンドリング

    let issues = response.json::<Vec<Issue>>().await?;

    Ok(issues)
}
