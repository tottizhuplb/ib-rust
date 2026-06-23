//! 订阅控制平面：desired → active 对齐。
//!
//! 对外 API：
//! - [`SubscriptionManager`]

mod manager;
mod registry;

pub use manager::SubscriptionManager;
