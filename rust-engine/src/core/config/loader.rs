use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::core::config::Config;
use crate::core::model::{LogRotation, LoggingConfig};
use crate::market::config::{
    resolve_port, AccountMode, IbConfig, MarketConfig, PipelineConfig, StorageConfig,
};
use crate::market::subscription::{DesiredSubscription, SubscriptionEntry};

const DEFAULT_CONFIG_PATH: &str = "conf/config.yaml";
const DEFAULT_SUBSCRIPTIONS_PATH: &str = "market/subscriptions.yaml";

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    ib: Option<FileIbConfig>,
    storage: Option<FileStorageConfig>,
    pipeline: Option<FilePipelineConfig>,
    logging: Option<FileLoggingConfig>,
    paths: Option<FilePathsConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct FileIbConfig {
    host: Option<String>,
    port: Option<u16>,
    client_id: Option<i32>,
}

#[derive(Debug, Default, Deserialize)]
struct FileStorageConfig {
    data_dir: Option<PathBuf>,
    segment_max_bytes: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct FilePipelineConfig {
    event_channel_capacity: Option<usize>,
    snapshot_channel_capacity: Option<usize>,
    flush_interval_ms: Option<u64>,
    snapshot_interval_secs: Option<u64>,
    reconnect_backoff_secs: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct FileLoggingConfig {
    log_dir: Option<PathBuf>,
    level: Option<String>,
    rotation: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FilePathsConfig {
    subscriptions: Option<PathBuf>,
}

pub fn load() -> anyhow::Result<Config> {
    let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.into());
    let config_path = PathBuf::from(&path);

    let (mut config, subscriptions_path) = if config_path.exists() {
        from_yaml(&config_path)?
    } else {
        tracing::info!(path = %path, "config file not found, using env defaults");
        (
            Config {
                logging: LoggingConfig::from_env(),
                market: MarketConfig::from_env(),
            },
            subscriptions_path_from(&config_path, None),
        )
    };

    config.market.subscriptions = load_subscriptions(&subscriptions_path)?;
    apply_env_overrides(&mut config);
    Ok(config)
}

fn from_yaml(path: &Path) -> anyhow::Result<(Config, PathBuf)> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read config file {}", path.display()))?;
    let file: FileConfig = serde_yaml::from_str(&text).context("parse config yaml")?;

    let account_mode = AccountMode::from_env(
        &std::env::var("TRADING_MODE").unwrap_or_else(|_| "paper".into()),
    );

    let ib_file = file.ib.unwrap_or_default();
    let host = ib_file.host.unwrap_or_else(|| "ib-gateway".into());
    let port = ib_file
        .port
        .unwrap_or_else(|| resolve_port(&host, account_mode, None));

    let subscriptions_path = subscriptions_path_from(path, file.paths.as_ref());

    let storage_file = file.storage.unwrap_or_default();
    let pipeline_file = file.pipeline.unwrap_or_default();
    let logging_file = file.logging.unwrap_or_default();

    Ok((
        Config {
            logging: logging_from_file(&logging_file),
            market: MarketConfig {
                ib: IbConfig {
                    host,
                    port,
                    client_id: ib_file.client_id.unwrap_or(101),
                    account_mode,
                },
                storage: StorageConfig {
                    data_dir: storage_file
                        .data_dir
                        .unwrap_or_else(|| PathBuf::from("./data")),
                    segment_max_bytes: storage_file.segment_max_bytes.unwrap_or(256 * 1024 * 1024),
                },
                pipeline: PipelineConfig {
                    event_channel_capacity: pipeline_file
                        .event_channel_capacity
                        .unwrap_or(50_000),
                    snapshot_channel_capacity: pipeline_file
                        .snapshot_channel_capacity
                        .unwrap_or(1_000),
                    flush_interval_ms: pipeline_file.flush_interval_ms.unwrap_or(500),
                    snapshot_interval_secs: pipeline_file.snapshot_interval_secs.unwrap_or(30),
                    reconnect_backoff_secs: pipeline_file.reconnect_backoff_secs.unwrap_or(1),
                },
                subscriptions: Vec::new(),
            },
        },
        subscriptions_path,
    ))
}

fn apply_env_overrides(config: &mut Config) {
    let env_market = MarketConfig::from_env();
    let env_logging = LoggingConfig::from_env();

    if std::env::var("IB_HOST").is_ok() {
        config.market.ib.host = env_market.ib.host;
    }
    if std::env::var("IB_PORT").is_ok() {
        config.market.ib.port = env_market.ib.port;
    }
    if std::env::var("IB_CLIENT_ID").is_ok() {
        config.market.ib.client_id = env_market.ib.client_id;
    }
    if std::env::var("TRADING_MODE").is_ok() {
        config.market.ib.account_mode = env_market.ib.account_mode;
        if std::env::var("IB_PORT").is_err() {
            config.market.ib.port = env_market.ib.port;
        }
    }
    if std::env::var("STORAGE_DATA_DIR").is_ok() {
        config.market.storage.data_dir = env_market.storage.data_dir;
    }
    if std::env::var("STORAGE_SEGMENT_MAX_BYTES").is_ok() {
        config.market.storage.segment_max_bytes = env_market.storage.segment_max_bytes;
    }
    if std::env::var("EVENT_CHANNEL_CAPACITY").is_ok() {
        config.market.pipeline.event_channel_capacity = env_market.pipeline.event_channel_capacity;
    }
    if std::env::var("SNAPSHOT_CHANNEL_CAPACITY").is_ok() {
        config.market.pipeline.snapshot_channel_capacity =
            env_market.pipeline.snapshot_channel_capacity;
    }
    if std::env::var("FLUSH_INTERVAL_MS").is_ok() {
        config.market.pipeline.flush_interval_ms = env_market.pipeline.flush_interval_ms;
    }
    if std::env::var("SNAPSHOT_INTERVAL_SECS").is_ok() {
        config.market.pipeline.snapshot_interval_secs = env_market.pipeline.snapshot_interval_secs;
    }
    if std::env::var("RECONNECT_BACKOFF_SECS").is_ok() {
        config.market.pipeline.reconnect_backoff_secs = env_market.pipeline.reconnect_backoff_secs;
    }
    if std::env::var("LOG_DIR").is_ok() {
        config.logging.log_dir = env_logging.log_dir;
    }
    if std::env::var("LOG_LEVEL").is_ok() {
        config.logging.level = env_logging.level;
    }
    if std::env::var("LOG_ROTATION").is_ok() {
        config.logging.rotation = env_logging.rotation;
    }
}

fn subscriptions_path_from(config_path: &Path, paths: Option<&FilePathsConfig>) -> PathBuf {
    if let Ok(path) = std::env::var("SUBSCRIPTIONS_PATH") {
        return PathBuf::from(path);
    }

    let relative = paths
        .and_then(|paths| paths.subscriptions.clone())
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SUBSCRIPTIONS_PATH));

    resolve_path(config_path, &relative)
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

fn logging_from_file(file: &FileLoggingConfig) -> LoggingConfig {
    let defaults = LoggingConfig::from_env();

    LoggingConfig {
        log_dir: file.log_dir.clone().unwrap_or(defaults.log_dir),
        level: file.level.clone().unwrap_or(defaults.level),
        rotation: file
            .rotation
            .as_deref()
            .map(LogRotation::from_env)
            .unwrap_or(defaults.rotation),
    }
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
