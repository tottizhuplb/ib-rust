//! 统一 WAL：event / snapshot 交错写入，全局 seq。
//!
//! 对外 API：
//! - [`RecorderService::run`]
//! - market 域 WAL：[`crate::market::wal`]

mod service;

pub use service::RecorderService;
