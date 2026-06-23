use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    Daily,
    Hourly,
}
