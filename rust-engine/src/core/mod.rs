//! 跨模块契约：事件 schema、pipeline trait、共享运行状态。
//!
//! 对外 API：
//! - [`RunState`]
//! - [`model`] — `MarketEvent`、`Symbol` 等共享类型
//! - [`config`] — yaml/env 加载
//! - [`pipeline`] — `EventPublisher`、channel 辅助、服务 trait
//!
//! 约定：不放 I/O、不放业务编排、不放与契约无关的 helper。

pub mod config;
pub mod model;
pub mod pipeline;
pub mod run_state;
pub mod task;

pub use config::Config;
pub use run_state::RunState;
pub use task::{EngineStop, StopReason, TaskGroup, TaskResult, wait_for_signal_or_worker};
