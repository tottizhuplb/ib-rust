use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub log_dir: PathBuf,
    pub level: String,
    pub rotation: LogRotation,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("./logs"),
            level: "info".into(),
            rotation: LogRotation::Daily,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    Daily,
    Hourly,
}

impl LogRotation {
    pub(crate) fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "hourly" => Self::Hourly,
            _ => Self::Daily,
        }
    }
}
