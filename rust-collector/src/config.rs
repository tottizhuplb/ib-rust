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
        let account_mode = AccountMode::from_env(
            &env::var("IB_ACCOUNT_MODE").unwrap_or_else(|_| "paper".into()),
        );

        let port = env::var("IB_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| account_mode.default_port());

        Self {
            host: env::var("IB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
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
}
