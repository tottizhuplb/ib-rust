//! 周期性导出内存盘口状态。
//!
//! 对外 API：
//! - [`SnapshotService::run`]

mod service;

pub use service::SnapshotService;
