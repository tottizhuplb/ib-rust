mod core;
mod logging;
mod market;
mod order;
mod risk;
mod strategy;

use anyhow::Result;
use tokio::sync::broadcast;
use tracing::info;

use crate::core::task::{wait_for_signal_or_worker, TaskGroup};

#[tokio::main]
async fn main() -> Result<()> {
    let config = core::Config::load()?;
    let _logging = logging::init(&config.logging)?;

    run(config).await
}

async fn run(config: core::Config) -> Result<()> {
    let (shutdown_tx, _) = broadcast::channel::<()>(16);
    let mut tasks = TaskGroup::new();

    let market = market::register(&mut tasks, &shutdown_tx, config.market)?;

    let stop = wait_for_signal_or_worker(wait_for_shutdown_signal(), &mut tasks).await?;

    info!(reason = ?stop.reason, "initiating graceful shutdown");
    market.begin_shutdown(&shutdown_tx);
    tasks.drain().await;

    info!("rust-engine stopped");
    stop.into_result()
}

async fn wait_for_shutdown_signal() -> Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut term = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
    }

    Ok(())
}
