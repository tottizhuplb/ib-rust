//! market 域内编排阶段（`watch` 广播，非落盘事件）。
//!
//! 描述 IB 连接 / 订阅协调的当前阶段；strategy / risk 不使用此类型。
//! 与落盘的 [`ConnectionEvent`](crate::core::model::ConnectionEvent) 职责不同。

/// market 域当前编排阶段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarketPhase {
    /// `register()` 创建 `watch`  channel 时的初始值；无 worker 专门处理。
    Starting,
    /// ConnectionManager 发起连接前；SubscriptionManager 清空 active 订阅。
    Connecting,
    /// TCP 已连上（尚未等同 session ready）；SubscriptionManager reconcile 订阅。
    Connected,
    /// 断线或连失败、退避重连中；SubscriptionManager 清空 active 订阅。
    Recovering,
    /// `MarketHandles::begin_shutdown` 写入；worker 实际靠 `broadcast` shutdown 退出。
    ShuttingDown,
}
