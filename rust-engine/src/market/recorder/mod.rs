//! 原始事件持久化：批量、flush、分段 jsonl.zst 文件。
//!
//! 对外 API：
//! - [`RecorderService::run`]
//! - [`JsonlZstdRecorder`]

mod service;
mod storage;

pub use service::RecorderService;
pub use storage::JsonlZstdRecorder;
