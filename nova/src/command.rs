mod ping;

use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::{
    application::interaction::Interaction, gateway::payload::incoming::MessageCreate,
};

use crate::Context;

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
}

pub async fn handle_message(event: Event, ctx: Arc<Context>) -> anyhow::Result<()> {
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

    if ping::Ping::NAMES.contains(&command) {
        let prefix_ctx = PrefixContext {
            msg: &msg,
            core: ctx,
        };
        ping::Ping::run_msg(prefix_ctx).await?;
    }

    Ok(())
}

