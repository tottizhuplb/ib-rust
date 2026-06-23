//! 跨模块契约：事件 schema、顶层 task 编排。
//!
//! 对外 API：
//! - [`model`] — `MarketEvent`、`Symbol` 等共享类型
//! - [`config`] — yaml/env 加载
//! - [`task`] — 顶层 TaskGroup、graceful shutdown
//!
//! 约定：不放 I/O、不放域内编排状态、不放与契约无关的 helper。

pub mod config;
pub mod model;
pub mod task;
pub mod wal;

pub use config::Config;
pub use task::{wait_for_signal_or_worker, EngineStop, StopReason, TaskGroup, TaskResult};
pub use wal::WalConfig;
