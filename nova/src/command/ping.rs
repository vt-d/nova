use twilight_model::{
    application::command::CommandType,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

use crate::command::{InteractionContext, InteractionRunnable, PrefixContext, PrefixRunnable};

pub struct Ping;

impl PrefixRunnable for Ping {
    const NAMES: &'static [&'static str] = &["ping"];

    async fn run_msg(ctx: PrefixContext<'_>) -> anyhow::Result<()> {
        ctx.core
            .client
            .create_message(ctx.msg.channel_id)
            .content("Pong!")
            .reply(ctx.msg.id)
            .await?;

        Ok(())
    }
}

impl InteractionRunnable for Ping {
    const NAME: &'static str = "ping";
    async fn run_interaction(ctx: InteractionContext<'_>) -> anyhow::Result<()> {
        let client = ctx.core.client.interaction(ctx.interaction.application_id);

        client
            .create_response(
                ctx.interaction.id,
                &ctx.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .content("Pong!")
                            .build(),
                    ),
                },
            )
            .await?;

        Ok(())
    }

    async fn create_command() -> anyhow::Result<twilight_model::application::command::Command> {
        Ok(CommandBuilder::new(Ping::NAME, "Pong!", CommandType::ChatInput).build())
    }
}
