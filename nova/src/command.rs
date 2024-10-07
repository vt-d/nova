pub mod info;
pub mod ping;

use anyhow::bail;
use std::mem;
use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::{
    application::{
        command::Command,
        interaction::{application_command::CommandData, Interaction, InteractionData},
    },
    gateway::payload::incoming::MessageCreate,
};

use crate::model::Context;

pub struct PrefixContext<'a> {
    pub msg: &'a MessageCreate,
    pub core: Arc<Context>,
}

pub trait PrefixRunnable: Sized {
    const NAMES: &'static [&'static str];

    async fn run_msg(ctx: PrefixContext<'_>) -> anyhow::Result<()>;
}

pub struct InteractionContext<'a> {
    pub interaction: &'a Interaction,
    pub core: Arc<Context>,
}

pub trait InteractionRunnable: Sized {
    const NAME: &'static str;

    async fn run_interaction(ctx: InteractionContext<'_>) -> anyhow::Result<()>;
    async fn create_command() -> anyhow::Result<Command>;
}

pub async fn handle_command(event: &mut Event, ctx: Arc<Context>) -> anyhow::Result<()> {
    handle_message(event, Arc::clone(&ctx)).await?;
    handle_interactions(event, Arc::clone(&ctx)).await?;
    Ok(())
}

pub async fn handle_message(event: &Event, ctx: Arc<Context>) -> anyhow::Result<()> {
    let msg = match event {
        Event::MessageCreate(msg) => msg,
        _ => return Ok(()),
    };

    let command = ctx
        .config
        .prefixes
        .iter()
        .find_map(|prefix| {
            msg.content
                .strip_prefix(prefix)
                .map(|cmd| cmd.split_whitespace().next())
        })
        .flatten();

    let command = match command {
        Some(cmd) => cmd,
        None => return Ok(()),
    };

    match command {
        cmd if ping::Ping::NAMES.contains(&cmd) => {
            let prefix_ctx = PrefixContext { msg, core: ctx };
            ping::Ping::run_msg(prefix_ctx).await?;
        }
        cmd if info::Info::NAMES.contains(&cmd) => {
            let prefix_ctx = PrefixContext { msg, core: ctx };
            info::Info::run_msg(prefix_ctx).await?;
        }
        _ => {}
    }

    Ok(())
}

pub async fn handle_interactions(event: &mut Event, client: Arc<Context>) -> anyhow::Result<()> {
    let interaction = match event {
        Event::InteractionCreate(interaction) => &mut interaction.0,
        _ => return Ok(()),
    };

    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        _ => {
            tracing::warn!("ignoring non-command interaction");
            return Ok(());
        }
    };

    start_interaction(interaction, data, client).await?;
    Ok(())
}

async fn start_interaction(
    interaction: &Interaction,
    data: CommandData,
    client: Arc<Context>,
) -> anyhow::Result<()> {
    let ctx = InteractionContext {
        core: client,
        interaction,
    };
    match &*data.name {
        ping::Ping::NAME => ping::Ping::run_interaction(ctx).await,
        info::Info::NAME => info::Info::run_interaction(ctx).await,
        name => bail!("unknown command: {}", name),
    }
}
