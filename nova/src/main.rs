mod command;

use command::handle_message;
use dotenvy::dotenv;
use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use twilight_gateway::{
    error::ReceiveMessageErrorType, CloseFrame, Event, EventTypeFlags, Intents, Shard,
    StreamExt as _,
};
use twilight_http::Client;
use twilight_model::{application::command::Command, oauth::Application};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub struct Config {
    pub token: String,
    pub prefixes: Vec<String>,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            token: env::var("DISCORD_TOKEN")?,
            prefixes: env::var("PREFIXES")?
                .split(", ")
                .map(String::from)
                .collect(),
        })
    }
}

pub struct Context {
    pub client: Arc<Client>,
    pub commands: Vec<Command>,
    pub application: Application,
    pub config: Config,
}

impl Context {
    pub async fn new(config: Config) -> anyhow::Result<Arc<Self>> {
        let client = Arc::new(Client::new(config.token.clone()));
        let application = client.current_user_application().await?.model().await?;
        Ok(Arc::new(Self {
            client,
            commands: vec![],
            application,
            config,
        }))
    }

    pub async fn create_shards(&self) -> anyhow::Result<Vec<Shard>> {
        let shards = twilight_gateway::create_recommended(
            &self.client,
            twilight_gateway::Config::new(
                self.config.token.clone(),
                Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
            ),
            |_, builder| builder.build(),
        )
        .await?
        .collect::<Vec<_>>();

        Ok(shards)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
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
        tasks.push(tokio::spawn(runner(shard, ctx_clone)));
    }

    tokio::signal::ctrl_c().await?;
    SHUTDOWN.store(true, Ordering::Relaxed);

    Ok(())
}

async fn runner(mut shard: Shard, ctx: Arc<Context>) {
    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        match event {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(event) => {
                tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "Received event");
                if let Err(e) = handle_message(event, Arc::clone(&ctx)).await {
                    tracing::warn!(?e, "Error handling message");
                }
            }
            Err(error)
                if SHUTDOWN.load(Ordering::Relaxed)
                    && matches!(error.kind(), ReceiveMessageErrorType::WebSocket) =>
            {
                break
            }
            Err(error) => {
                tracing::warn!(?error, "Error while receiving event");
            }
        }
    }

    shard.close(CloseFrame::NORMAL);
}
