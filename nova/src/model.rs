use std::{env, sync::Arc};

use twilight_gateway::{Intents, Shard};
use twilight_http::Client;
use twilight_model::{application::command::Command, oauth::Application};

use crate::command::{self, InteractionRunnable};

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
            commands: vec![command::ping::Ping::create_command().await?],
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

pub struct Config {
    pub token: String,
    pub prefixes: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            token: env::var("DISCORD_TOKEN")?,
            prefixes: env::var("PREFIXES")?
                .split(", ")
                .map(String::from)
                .collect(),
        })
    }
}
