//! 静态配置：解析后的模型 + yaml/env 加载。
//!
//! 对外 API：
//! - [`Config::load`] — yaml + 环境变量覆盖
//! - [`Config::from_env`] — 仅环境变量默认值
//! - [`Config`]、[`IbConfig`]、[`StorageConfig`]、[`PipelineConfig`]、[`AccountMode`]

mod loader;
mod model;

pub use model::{AccountMode, Config, IbConfig, PipelineConfig, StorageConfig};
