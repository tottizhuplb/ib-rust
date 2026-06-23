//! 跨模块契约：事件 schema、pipeline trait、顶层 task 编排。
//!
//! 对外 API：
//! - [`model`] — `MarketEvent`、`Symbol` 等共享类型
//! - [`config`] — yaml/env 加载
//! - [`pipeline`] — `EventPublisher`、channel 辅助、服务 trait
//! - [`task`] — 顶层 TaskGroup、graceful shutdown
//!
//! 约定：不放 I/O、不放域内编排状态、不放与契约无关的 helper。

pub mod config;
pub mod model;
pub mod pipeline;
pub mod task;

pub use config::Config;
pub use task::{wait_for_signal_or_worker, EngineStop, StopReason, TaskGroup, TaskResult};
