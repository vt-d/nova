mod command;
mod gateway;
mod model;
pub mod nova;
mod runner;

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    nova::run().await?;

    Ok(())
}
