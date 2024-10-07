use std::sync::{atomic::AtomicBool, Arc};

use twilight_gateway::{CloseFrame, Shard};

use crate::{gateway::process, model::Context};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn runner(mut shard: Shard, ctx: Arc<Context>) -> anyhow::Result<()> {
    tracing::info!("Initilized shard {}", shard.id().number());
    process(&mut shard, ctx).await?;
    shard.close(CloseFrame::NORMAL);
    Ok(())
}
