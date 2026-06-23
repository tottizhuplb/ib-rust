//! 统一 WAL 写端：event / snapshot 交错写入，全局 seq。
//!
//! 对外 API：
//! - [`RecorderService::run`] — 消费 event、维护 state、周期性 snapshot
//! - WAL 类型：[`MarketWalWriter`]、[`MarketWalReader`]、[`MarketWalRecord`]

mod service;
mod wal;

pub use service::RecorderService;
pub use wal::{MarketWalReader, MarketWalRecord, MarketWalWriter};
