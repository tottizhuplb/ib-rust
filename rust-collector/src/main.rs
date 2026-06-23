mod config;

use std::time::Duration;

use config::Config;
use ibapi::prelude::*;
use tokio::time::sleep;
use tracing::{info, warn};

const MAX_CONNECT_ATTEMPTS: u32 = 30;
const CONNECT_RETRY_DELAY: Duration = Duration::from_secs(2);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    info!(
        host = %config.host,
        port = config.port,
        client_id = config.client_id,
        account_mode = ?config.account_mode,
        "starting rust-collector"
    );

    let client = connect_with_retry(&config).await;
    info!(url = %config.connection_url(), "connected to IB Gateway");

    if let Ok(accounts) = client.managed_accounts().await {
        info!(?accounts, "managed accounts");
    }

    info!("collector idle; press Ctrl+C to stop");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for shutdown signal");
    info!("shutting down");
}

async fn connect_with_retry(config: &Config) -> Client {
    let url = config.connection_url();

    for attempt in 1..=MAX_CONNECT_ATTEMPTS {
        match Client::connect(&url, config.client_id).await {
            Ok(client) => return client,
            Err(error) => {
                warn!(
                    attempt,
                    max_attempts = MAX_CONNECT_ATTEMPTS,
                    %error,
                    "IB Gateway connection failed; retrying"
                );

                if attempt == MAX_CONNECT_ATTEMPTS {
                    panic!("failed to connect to IB Gateway at {url}: {error}");
                }

                sleep(CONNECT_RETRY_DELAY).await;
            }
        }
    }

    unreachable!("connect loop exits only on success or panic");
}
