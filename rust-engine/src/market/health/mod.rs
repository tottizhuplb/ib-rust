//! 进程健康与运行状态可观测性。
//!
//! 对外 API：
//! - [`HealthService::run`]

mod service;

pub use service::HealthService;
