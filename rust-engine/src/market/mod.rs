//! 行情域：IB 连接、订阅、状态、落盘、快照、健康检查。
//!
//! 数据流（当前阶段）：
//! ```text
//! IB Gateway → connection → subscription → (events) → recorder / state → snapshot
//! ```
//!
//! 对外 API（由 `app.rs` 使用）：
//! - [`connection::ConnectionManager`]
//! - [`connection::IbGatewayClient`]
//! - [`subscription::SubscriptionManager`]
//! - [`recorder::RecorderService`]
//! - [`state::OrderBookStore`]
//! - [`snapshot::SnapshotService`]
//! - [`health::HealthService`]

pub mod connection;
pub mod health;
pub mod recorder;
pub mod snapshot;
pub mod state;
pub mod subscription;

pub use connection::{ConnectionManager, IbGatewayClient};
pub use health::HealthService;
pub use recorder::{JsonlZstdRecorder, RecorderService};
pub use snapshot::SnapshotService;
pub use state::OrderBookStore;
pub use subscription::SubscriptionManager;
