mod app;
mod core;
mod logging;
mod market;
mod order;
mod risk;
mod strategy;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = core::Config::load()?;
    let _logging = logging::init(&config.logging)?;

    app::App::new(config).run().await
}
