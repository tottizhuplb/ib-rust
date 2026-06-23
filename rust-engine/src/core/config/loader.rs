use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::core::config::Config;
use crate::core::model::LoggingConfig;
use crate::market::config::{
    resolve_port, AccountMode, IbConfig, MarketConfig, PipelineConfig, StorageConfig,
};
use crate::market::subscription::{DesiredSubscription, SubscriptionEntry};

const CONFIG_PATH: &str = "conf/config.yaml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    ib: IbSection,
    storage: StorageConfig,
    pipeline: PipelineConfig,
    logging: LoggingConfig,
    paths: PathsSection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IbSection {
    client_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PathsSection {
    subscriptions: PathBuf,
}

pub fn load() -> anyhow::Result<Config> {
    let config_path = Path::new(CONFIG_PATH);
    if !config_path.exists() {
        anyhow::bail!("config file not found: {}", config_path.display());
    }

    let (mut config, subscriptions_path) = from_yaml(config_path)?;
    config.market.subscriptions = load_subscriptions(&subscriptions_path)?;
    Ok(config)
}

fn from_yaml(path: &Path) -> anyhow::Result<(Config, PathBuf)> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read config file {}", path.display()))?;
    let file: FileConfig = serde_yaml::from_str(&text).context("parse config yaml")?;

    let port = resolve_port(account_mode_from_env()?);
    let subscriptions_path = resolve_path(path, &file.paths.subscriptions);

    Ok((
        Config {
            logging: file.logging,
            market: MarketConfig {
                ib: IbConfig {
                    port,
                    client_id: file.ib.client_id,
                },
                storage: file.storage,
                pipeline: file.pipeline,
                subscriptions: Vec::new(),
            },
        },
        subscriptions_path,
    ))
}

fn account_mode_from_env() -> anyhow::Result<AccountMode> {
    let mode = std::env::var("TRADING_MODE")
        .context("TRADING_MODE must be set in environment (see .env)")?;
    Ok(AccountMode::parse(&mode))
}

fn resolve_path(base_config: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    base_config
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(path)
}

fn load_subscriptions(path: &Path) -> anyhow::Result<Vec<DesiredSubscription>> {
    if !path.exists() {
        tracing::info!(path = %path.display(), "subscriptions file not found, starting with none");
        return Ok(Vec::new());
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read subscriptions file {}", path.display()))?;
    let entries: Vec<SubscriptionEntry> =
        serde_yaml::from_str(&text).context("parse subscriptions yaml")?;

    let mut subscriptions = Vec::new();
    for entry in entries {
        subscriptions.extend(entry.expand_desired()?);
    }

    tracing::info!(path = %path.display(), count = subscriptions.len(), "loaded subscriptions");

    Ok(subscriptions)
}
