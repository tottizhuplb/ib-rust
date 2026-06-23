use std::path::PathBuf;

mod file;

#[derive(Debug, Clone)]
pub struct Config {
    pub ib: IbConfig,
    pub storage: StorageConfig,
    pub pipeline: PipelineConfig,
    pub subscriptions: Vec<crate::domain::DesiredSubscription>,
}

#[derive(Debug, Clone)]
pub struct IbConfig {
    pub host: String,
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

impl Config {
    pub fn from_env() -> Self {
        use std::env;

        let account_mode =
            AccountMode::from_env(&env::var("TRADING_MODE").unwrap_or_else(|_| "paper".into()));

        let host = env::var("IB_HOST").unwrap_or_else(|_| "ib-gateway".into());
        let explicit_port = env::var("IB_PORT")
            .ok()
            .and_then(|value| value.parse().ok());
        let port = resolve_port(&host, account_mode, explicit_port);

        Self {
            ib: IbConfig {
                host,
                port,
                client_id: env::var("IB_CLIENT_ID")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(101),
                account_mode,
            },
            storage: StorageConfig {
                data_dir: env::var("STORAGE_DATA_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("./data")),
                segment_max_bytes: env::var("STORAGE_SEGMENT_MAX_BYTES")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(256 * 1024 * 1024),
            },
            pipeline: PipelineConfig {
                event_channel_capacity: env::var("EVENT_CHANNEL_CAPACITY")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(50_000),
                snapshot_channel_capacity: env::var("SNAPSHOT_CHANNEL_CAPACITY")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1_000),
                flush_interval_ms: env::var("FLUSH_INTERVAL_MS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(500),
                snapshot_interval_secs: env::var("SNAPSHOT_INTERVAL_SECS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(30),
                reconnect_backoff_secs: env::var("RECONNECT_BACKOFF_SECS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1),
            },
            subscriptions: Vec::new(),
        }
    }
}

impl IbConfig {
    pub fn connection_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountMode {
    Paper,
    Live,
}

impl AccountMode {
    pub(crate) fn from_env(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "live" => Self::Live,
            _ => Self::Paper,
        }
    }

    pub fn default_port(self) -> u16 {
        match self {
            Self::Paper => 4002,
            Self::Live => 4001,
        }
    }

    pub fn gnzsnz_docker_port(self) -> u16 {
        match self {
            Self::Paper => 4004,
            Self::Live => 4003,
        }
    }
}

pub(crate) fn resolve_port(
    host: &str,
    account_mode: AccountMode,
    explicit_port: Option<u16>,
) -> u16 {
    if let Some(port) = explicit_port {
        return port;
    }

    if host == "ib-gateway" {
        return account_mode.gnzsnz_docker_port();
    }

    account_mode.default_port()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ib_gateway_host_uses_gnzsnz_docker_port() {
        assert_eq!(resolve_port("ib-gateway", AccountMode::Paper, None), 4004);
    }
}
