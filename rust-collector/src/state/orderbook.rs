use crate::domain::OrderBookSnapshot;

/// In-memory order books keyed by symbol; updated by OrderBookService.
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
