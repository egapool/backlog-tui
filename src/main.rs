mod app;
mod client;
mod crossterm;
mod ui;

use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    crossterm::run().await?;
    Ok(())
}
