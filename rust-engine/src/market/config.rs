use std::path::PathBuf;

use crate::core::wal::WalConfig;
use crate::market::subscription::DesiredSubscription;

/// Docker Compose 服务名，与 `docker-compose.yml` 中 ib-gateway 一致，不可配置。
pub const IB_GATEWAY_HOST: &str = "ib-gateway";

/// market 域 WAL 子目录名（`{data_dir}/market`）。
pub const MARKET_WAL_DOMAIN: &str = "market";

#[derive(Debug, Clone)]
pub struct MarketConfig {
    pub ib: IbConfig,
    pub storage: StorageConfig,
    pub pipeline: PipelineConfig,
    pub subscriptions: Vec<DesiredSubscription>,
}

#[derive(Debug, Clone)]
pub struct IbConfig {
    pub port: u16,
    pub client_id: i32,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// WAL 根目录，各域使用 `{root_dir}/{domain}` 子目录。
    pub root_dir: PathBuf,
}

impl StorageConfig {
    pub fn wal_config(&self) -> WalConfig {
        WalConfig {
            root_dir: self.root_dir.clone(),
            domain: MARKET_WAL_DOMAIN,
        }
    }

    pub fn wal_data_dir(&self) -> PathBuf {
        self.wal_config().data_dir()
    }
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub event_channel_capacity: usize,
    pub flush_interval_ms: u64,
    pub snapshot_interval_secs: u64,
    pub reconnect_backoff_secs: u64,
}

impl IbConfig {
    pub fn connection_url(&self) -> String {
        format!("{IB_GATEWAY_HOST}:{}", self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountMode {
    Paper,
    Live,
}

impl AccountMode {
    pub(crate) fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "live" => Self::Live,
            _ => Self::Paper,
        }
    }

    pub fn gnzsnz_docker_port(self) -> u16 {
        match self {
            Self::Paper => 4004,
            Self::Live => 4003,
        }
    }
}

pub(crate) fn resolve_port(account_mode: AccountMode) -> u16 {
    account_mode.gnzsnz_docker_port()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_mode_uses_gnzsnz_docker_port() {
        assert_eq!(resolve_port(AccountMode::Paper), 4004);
    }
}
