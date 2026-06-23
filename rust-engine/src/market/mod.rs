//! 行情域：IB 连接、订阅、状态、落盘、快照、健康检查。
//!
//! 数据流（当前阶段）：
//! ```text
//! IB Gateway → connection → subscription → (events) → recorder / state → snapshot
//! ```
//!
//! 对外 API（由 `main.rs` 使用）：
//! - [`register`] — 向顶层 [`TaskGroup`](crate::core::task::TaskGroup) 注册本域 worker
//! - [`MarketHandles::begin_shutdown`] — 本域状态与 event channel 收尾

pub mod config;
pub mod connection;
pub mod health;
pub mod recorder;
pub mod runtime;
pub mod snapshot;
pub mod state;
pub mod subscription;

pub use connection::{ConnectionManager, IbGatewayClient};
pub use health::HealthService;
pub use recorder::{JsonlZstdRecorder, RecorderService};
pub use runtime::{register, MarketHandles};
pub use snapshot::SnapshotService;
pub use state::OrderBookStore;
pub use subscription::SubscriptionManager;
