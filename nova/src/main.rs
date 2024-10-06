mod command;

use std::{
    env,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use twilight_gateway::{
    error::ReceiveMessageErrorType, CloseFrame, Event, EventTypeFlags, Intents, Shard,
    StreamExt as _,
};
use twilight_http::Client;
use twilight_model::{application::command::Command, oauth::Application};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

struct Config {
    pub token: String,
    pub prefixes: Vec<String>,
}

impl Config {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            token: env::var("DISCORD_TOKEN")?,
            prefixes: env::var("PREFIXES")?
                .split(", ")
                .map(String::from)
                .collect(),
        })
    }
}

pub struct ContextRef {
    pub client: Arc<Client>,
    pub commands: [Command; 0],
    pub application: Application,
    config: Config,
}

struct Context(Arc<ContextRef>);

impl Context {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let client = Arc::new(Client::new(config.token.clone()));
        let application = client.current_user_application().await?.model().await?;

        Ok(Self(Arc::new(ContextRef {
            client,
            commands: [],
            application,
            config,
        })))
    }

    pub async fn shards_create(&self) -> anyhow::Result<Vec<Shard>> {
        Ok(twilight_gateway::create_recommended(
            &self.client,
            twilight_gateway::Config::new(self.config.token.clone(), Intents::empty()),
            |_, builder| builder.build(),
        )
        .await?
        .collect())
    }
}

impl Deref for Context {
    type Target = Arc<ContextRef>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    let config = Config::new()?;

    let fmt_tracing_layer = tracing_subscriber::fmt::layer().without_time().pretty();
    tracing_subscriber::registry()
        .with(fmt_tracing_layer)
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .try_init()?;

    let ctx = Context::new(config).await?;
    let interaction_client = ctx.client.interaction(ctx.application.id);

    tracing::info!("Logged into application {}", ctx.application.name);

    if let Err(error) = interaction_client.set_global_commands(&ctx.commands).await {
        tracing::error!(?error, "Failed to register global commands");
    }

    let shards = ctx.shards_create().await?;

    let shard_len = shards.len();
    let mut senders = Vec::with_capacity(shard_len);
    let mut tasks = Vec::with_capacity(shard_len);

    for shard in shards {
        senders.push(shard.sender());
        tasks.push(tokio::spawn(runner(shard, ctx.client.clone())));
    }

    tokio::signal::ctrl_c().await?;
    SHUTDOWN.store(true, Ordering::Relaxed);
    for sender in senders {
        _ = sender.close(CloseFrame::NORMAL);
    }

    for jh in tasks {
        _ = jh.await;
    }

    Ok(())
}

async fn runner(mut shard: Shard, _: Arc<Client>) {
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(event) => event,
            Err(error)
                if SHUTDOWN.load(Ordering::Relaxed)
                    && matches!(error.kind(), ReceiveMessageErrorType::WebSocket) =>
            {
                break
            }
            Err(error) => {
                tracing::warn!(?error, "error while receiving event");
                continue;
            }
        };

        tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
    }
}
