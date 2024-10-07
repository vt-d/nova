use crate::model::{Config, Context};
use crate::runner::{self, SHUTDOWN};
use std::{
    env,
    sync::{atomic::Ordering, Arc},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub async fn run() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time().pretty())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .try_init()?;

    let ctx = Context::new(config).await?;
    let interaction_client = ctx.client.interaction(ctx.application.id);

    tracing::info!("Logged into application: {}", ctx.application.name);

    if let Err(error) = interaction_client.set_global_commands(&ctx.commands).await {
        tracing::error!(?error, "Failed to register global commands");
    }

    let shards = ctx.create_shards().await?;
    let mut tasks = Vec::with_capacity(shards.len());

    for shard in shards {
        let ctx_clone = Arc::clone(&ctx);
        tasks.push(tokio::spawn(runner::runner(shard, ctx_clone)));
    }

    tokio::signal::ctrl_c().await?;
    SHUTDOWN.store(true, Ordering::Relaxed);
    Ok(())
}
