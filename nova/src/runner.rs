use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use twilight_gateway::{
    error::ReceiveMessageErrorType, CloseFrame, Event, EventTypeFlags, Shard, StreamExt as _,
};

use crate::{command::handle_command, model::Context};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn runner(mut shard: Shard, ctx: Arc<Context>) -> anyhow::Result<()> {
    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        match event {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(mut event) => {
                tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "Received event");
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

    shard.close(CloseFrame::NORMAL);
    Ok(())
}
