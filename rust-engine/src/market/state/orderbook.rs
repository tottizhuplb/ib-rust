use crate::core::model::OrderBookSnapshot;

/// 按 symbol 索引的内存盘口；由 depth 事件处理器更新。
#[derive(Default)]
pub struct OrderBookStore;

impl OrderBookStore {
    pub fn new() -> Self {
        Self
    }

    pub async fn snapshot_all(&self) -> Vec<OrderBookSnapshot> {
        Vec::new()
    }
}
