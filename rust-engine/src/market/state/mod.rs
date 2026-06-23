//! 运行期内存状态（非配置、非持久化层）。
//!
//! 对外 API：
//! - [`OrderBookStore`] — 由 depth 事件维护的 L2 内存盘口

mod orderbook;

pub use orderbook::OrderBookStore;
