//! IB Gateway 连接 supervisor 与客户端适配层。
//!
//! 对外 API：
//! - [`ConnectionManager::run_supervisor`]
//! - [`IbGatewayClient`]
//! - [`IbSession`]

mod adapter;
mod client;
mod session;
mod supervisor;

pub use client::IbGatewayClient;
pub use session::IbSession;
pub use supervisor::ConnectionManager;
