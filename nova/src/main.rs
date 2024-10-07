pub mod nova;

mod command;
mod gateway;
mod model;
mod runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    nova::run().await?;

    Ok(())
}
