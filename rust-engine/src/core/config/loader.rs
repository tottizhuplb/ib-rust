use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::core::config::Config;
use crate::core::model::{LogRotation, LoggingConfig};
use crate::market::config::{
    resolve_port, AccountMode, IbConfig, MarketConfig, PipelineConfig, StorageConfig,
};
use crate::market::subscription::{DesiredSubscription, SubscriptionEntry};

const CONFIG_PATH: &str = "conf/config.yaml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    ib: FileIbConfig,
    storage: FileStorageConfig,
    pipeline: FilePipelineConfig,
    logging: FileLoggingConfig,
    paths: FilePathsConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileIbConfig {
    client_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileStorageConfig {
    data_dir: PathBuf,
    segment_max_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FilePipelineConfig {
    event_channel_capacity: usize,
    flush_interval_ms: u64,
    snapshot_interval_secs: u64,
    reconnect_backoff_secs: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileLoggingConfig {
    log_dir: PathBuf,
    level: String,
    rotation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FilePathsConfig {
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

    let account_mode = account_mode_from_env()?;
    let port = resolve_port(account_mode);
    let subscriptions_path = resolve_path(path, &file.paths.subscriptions);

    Ok((
        Config {
            logging: logging_from_file(&file.logging)?,
            market: MarketConfig {
                ib: IbConfig {
                    port,
                    client_id: file.ib.client_id,
                },
                storage: StorageConfig {
                    data_dir: file.storage.data_dir,
                    segment_max_bytes: file.storage.segment_max_bytes,
                },
                pipeline: PipelineConfig {
                    event_channel_capacity: file.pipeline.event_channel_capacity,
                    flush_interval_ms: file.pipeline.flush_interval_ms,
                    snapshot_interval_secs: file.pipeline.snapshot_interval_secs,
                    reconnect_backoff_secs: file.pipeline.reconnect_backoff_secs,
                },
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

fn logging_from_file(file: &FileLoggingConfig) -> anyhow::Result<LoggingConfig> {
    Ok(LoggingConfig {
        log_dir: file.log_dir.clone(),
        level: file.level.clone(),
        rotation: LogRotation::parse(&file.rotation),
    })
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

    tracing::info!(path = %path.display(), count = entries.len(), "loaded subscriptions");

    Ok(entries.into_iter().map(Into::into).collect())
}
