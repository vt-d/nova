use super::{PrefixContext, PrefixRunnable};

pub struct Ping;

impl PrefixRunnable for Ping {
    const NAMES: &'static [&'static str] = &["ping"];

    async fn run_msg(ctx: PrefixContext<'_>) -> anyhow::Result<()> {
        // Send a message to the channel, replying to the original message.
        ctx.core
            .client
            .create_message(ctx.msg.channel_id)
            .content("Pong!")
            .reply(ctx.msg.id)
            .await?;

        Ok(())
    }
}
