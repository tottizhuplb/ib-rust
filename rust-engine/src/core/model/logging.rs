use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub log_dir: PathBuf,
    pub level: String,
    pub rotation: LogRotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    Daily,
    Hourly,
}

impl LogRotation {
    pub(crate) fn from_env(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "hourly" => Self::Hourly,
            _ => Self::Daily,
        }
    }
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        use std::env;

        Self {
            log_dir: env::var("LOG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./logs")),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            rotation: env::var("LOG_ROTATION")
                .map(|value| LogRotation::from_env(&value))
                .unwrap_or(LogRotation::Daily),
        }
    }
}
