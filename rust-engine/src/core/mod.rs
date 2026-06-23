//! 跨模块契约：事件 schema、pipeline trait、共享运行状态。
//!
//! 对外 API：
//! - [`RunState`]
//! - [`domain`] — `MarketEvent`、`Symbol`、订阅类型等
//! - [`pipeline`] — `EventPublisher`、channel 辅助、服务 trait
//!
//! 约定：不放 I/O、不放业务编排、不放与契约无关的 helper。

pub mod domain;
pub mod pipeline;
pub mod run_state;

pub use run_state::RunState;
