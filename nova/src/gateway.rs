use std::sync::{atomic::Ordering, Arc};

use twilight_gateway::{
    error::ReceiveMessageErrorType, Event, EventTypeFlags, Shard, StreamExt as _,
};

use crate::runner::SHUTDOWN;
use crate::{command::handle_command, model::Context};

pub async fn process(shard: &mut Shard, ctx: Arc<Context>) -> anyhow::Result<()> {
    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        match event {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(mut event) => {
                handle_command(&mut event, Arc::clone(&ctx)).await?;
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

    Ok(())
}
