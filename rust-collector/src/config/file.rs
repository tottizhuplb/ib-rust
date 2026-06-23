use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::domain::{DesiredSubscription, SubscriptionEntry};

use super::{AccountMode, Config, IbConfig, PipelineConfig, StorageConfig};

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    ib: Option<FileIbConfig>,
    storage: Option<FileStorageConfig>,
    pipeline: Option<FilePipelineConfig>,
    subscriptions: Option<Vec<SubscriptionEntry>>,
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

impl Config {
    /// Load `config.yaml` (or `CONFIG_PATH`), then apply environment overrides.
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".into());
        let mut config = if Path::new(&path).exists() {
            Self::from_file(&path)?
        } else {
            tracing::info!(path = %path, "config file not found, using env defaults");
            Self::from_env()
        };
        config.apply_env_overrides();
        Ok(config)
    }

    fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("read config file {}", path.as_ref().display()))?;
        let file: FileConfig =
            serde_yaml::from_str(&text).context("parse config yaml")?;

        let account_mode = AccountMode::from_env(
            &std::env::var("TRADING_MODE").unwrap_or_else(|_| "paper".into()),
        );

        let ib_file = file.ib.unwrap_or_default();
        let host = ib_file
            .host
            .unwrap_or_else(|| "ib-gateway".into());
        let port = ib_file.port.unwrap_or_else(|| {
            super::resolve_port(&host, account_mode, None)
        });

        let subscriptions: Vec<DesiredSubscription> = file
            .subscriptions
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        let storage_file = file.storage.unwrap_or_default();
        let pipeline_file = file.pipeline.unwrap_or_default();

        Ok(Self {
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
                segment_max_bytes: storage_file
                    .segment_max_bytes
                    .unwrap_or(256 * 1024 * 1024),
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
            subscriptions,
        })
    }

    fn apply_env_overrides(&mut self) {
        let env_config = Self::from_env();
        if std::env::var("IB_HOST").is_ok() {
            self.ib.host = env_config.ib.host;
        }
        if std::env::var("IB_PORT").is_ok() {
            self.ib.port = env_config.ib.port;
        }
        if std::env::var("IB_CLIENT_ID").is_ok() {
            self.ib.client_id = env_config.ib.client_id;
        }
        if std::env::var("TRADING_MODE").is_ok() {
            self.ib.account_mode = env_config.ib.account_mode;
            if std::env::var("IB_PORT").is_err() {
                self.ib.port = env_config.ib.port;
            }
        }
        if std::env::var("STORAGE_DATA_DIR").is_ok() {
            self.storage.data_dir = env_config.storage.data_dir;
        }
        if std::env::var("STORAGE_SEGMENT_MAX_BYTES").is_ok() {
            self.storage.segment_max_bytes = env_config.storage.segment_max_bytes;
        }
        if std::env::var("EVENT_CHANNEL_CAPACITY").is_ok() {
            self.pipeline.event_channel_capacity = env_config.pipeline.event_channel_capacity;
        }
        if std::env::var("SNAPSHOT_CHANNEL_CAPACITY").is_ok() {
            self.pipeline.snapshot_channel_capacity =
                env_config.pipeline.snapshot_channel_capacity;
        }
        if std::env::var("FLUSH_INTERVAL_MS").is_ok() {
            self.pipeline.flush_interval_ms = env_config.pipeline.flush_interval_ms;
        }
        if std::env::var("SNAPSHOT_INTERVAL_SECS").is_ok() {
            self.pipeline.snapshot_interval_secs = env_config.pipeline.snapshot_interval_secs;
        }
        if std::env::var("RECONNECT_BACKOFF_SECS").is_ok() {
            self.pipeline.reconnect_backoff_secs = env_config.pipeline.reconnect_backoff_secs;
        }
    }
}
