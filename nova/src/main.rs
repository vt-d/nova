pub mod nova;

mod command;
mod gateway;
mod model;
mod runner;

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    nova::run().await?;

    Ok(())
}
