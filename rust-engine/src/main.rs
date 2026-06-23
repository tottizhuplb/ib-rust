mod app;
mod config;
mod core;
mod market;
mod order;
mod risk;
mod strategy;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::Config::load()?;
    app::App::new(config).run().await
}
