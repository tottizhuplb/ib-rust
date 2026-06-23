//! 静态配置：yaml/env 加载与 [`Config`] 模型。

mod loader;

use crate::core::model::LoggingConfig;
use crate::market::config::MarketConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub logging: LoggingConfig,
    pub market: MarketConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        loader::load()
    }
}
