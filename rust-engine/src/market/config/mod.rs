use std::path::PathBuf;

use crate::market::subscription::DesiredSubscription;

/// Docker Compose 服务名，与 `docker-compose.yml` 中 ib-gateway 一致，不可配置。
pub const IB_GATEWAY_HOST: &str = "ib-gateway";

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
    pub account_mode: AccountMode,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub segment_max_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub event_channel_capacity: usize,
    pub snapshot_channel_capacity: usize,
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
