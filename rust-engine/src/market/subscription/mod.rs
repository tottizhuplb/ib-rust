//! 订阅控制平面：desired → active 对齐。
//!
//! 对外 API：
//! - [`SubscriptionManager`]

mod manager;
mod model;
mod registry;

pub use manager::SubscriptionManager;
pub use model::{
    ActiveSubscription, DesiredSubscription, SubscriptionEntry, SubscriptionKey, SubscriptionKind,
    SubscriptionStatus,
};
