use std::path::Path;

use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::core::model::{LogRotation, LoggingConfig};

pub struct LoggingGuard {
    _file_guard: WorkerGuard,
}

pub fn init(config: &LoggingConfig) -> anyhow::Result<LoggingGuard> {
    std::fs::create_dir_all(&config.log_dir)
        .with_context(|| format!("create log dir {}", config.log_dir.display()))?;

    let filter = EnvFilter::try_new(&config.level)
        .with_context(|| format!("invalid log level `{}`", config.level))?;

    let appender = file_appender(&config.log_dir, config.rotation);
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);
    let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);

    Registry::default()
        .with(filter)
        .with(file_layer)
        .with(fmt::layer())
        .init();

    Ok(LoggingGuard {
        _file_guard: guard,
    })
}

fn file_appender(
    log_dir: &Path,
    rotation: LogRotation,
) -> tracing_appender::rolling::RollingFileAppender {
    match rotation {
        LogRotation::Daily => tracing_appender::rolling::daily(log_dir, "engine.log"),
        LogRotation::Hourly => tracing_appender::rolling::hourly(log_dir, "engine.log"),
    }
}
