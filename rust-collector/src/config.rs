use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountMode {
    Paper,
    Live,
}

impl AccountMode {
    fn from_env(value: &str) -> Self {
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

    /// gnzsnz/ib-gateway-docker 通过 socat 在容器网络暴露 API（paper=4004，live=4003）
    pub fn gnzsnz_docker_port(self) -> u16 {
        match self {
            Self::Paper => 4004,
            Self::Live => 4003,
        }
    }
}

fn resolve_port(host: &str, account_mode: AccountMode, explicit_port: Option<u16>) -> u16 {
    if let Some(port) = explicit_port {
        return port;
    }

    if host == "ib-gateway" {
        return account_mode.gnzsnz_docker_port();
    }

    account_mode.default_port()
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub client_id: i32,
    pub account_mode: AccountMode,
}

impl Config {
    pub fn from_env() -> Self {
        // 与 IB Gateway 共用 TRADING_MODE；Docker 网络内连 ib-gateway 时用 gnzsnz socat 端口
        let account_mode = AccountMode::from_env(
            &env::var("TRADING_MODE").unwrap_or_else(|_| "paper".into()),
        );

        let host = env::var("IB_HOST").unwrap_or_else(|_| "ib-gateway".into());
        let explicit_port = env::var("IB_PORT")
            .ok()
            .and_then(|value| value.parse().ok());
        let port = resolve_port(&host, account_mode, explicit_port);

        Self {
            host,
            port,
            client_id: env::var("IB_CLIENT_ID")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(101),
            account_mode,
        }
    }

    pub fn connection_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_mode_defaults_to_port_4002() {
        assert_eq!(AccountMode::Paper.default_port(), 4002);
    }

    #[test]
    fn live_mode_defaults_to_port_4001() {
        assert_eq!(AccountMode::Live.default_port(), 4001);
    }

    #[test]
    fn gnzsnz_docker_paper_port_is_4004() {
        assert_eq!(AccountMode::Paper.gnzsnz_docker_port(), 4004);
    }

    #[test]
    fn ib_gateway_host_uses_gnzsnz_docker_port() {
        assert_eq!(
            resolve_port("ib-gateway", AccountMode::Paper, None),
            4004
        );
    }

    #[test]
    fn localhost_uses_standard_ib_ports() {
        assert_eq!(resolve_port("127.0.0.1", AccountMode::Paper, None), 4002);
    }
}
