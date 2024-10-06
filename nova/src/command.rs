use twilight_model::{
    application::interaction::Interaction, gateway::payload::incoming::MessageCreate,
};

use crate::Context;

struct PrefixContext {
    pub msg: MessageCreate,
    pub core: Context,
}

trait PrefixRunnable: Sized {
    const NAMES: [&'static str];

    async fn run_msg(ctx: PrefixContext) -> anyhow::Result<()>;
}

struct InteractionContext {
    pub interaction: Interaction,
    pub core: Context,
}

trait InteractionRunnable: Sized {
    const NAME: &'static str;

    async fn run_interaction(ctx: InteractionContext) -> anyhow::Result<()>;
}




