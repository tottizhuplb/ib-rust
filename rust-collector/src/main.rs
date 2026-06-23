mod app;
mod config;
mod domain;
mod ib;
mod metrics;
mod pipeline;
mod services;
mod state;
mod storage;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env();
    app::App::new(config)?.run().await
}
