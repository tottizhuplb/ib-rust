use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::model::{
    BookLevel, BookSide, DepthEvent, DepthOperation, MarketEvent, OrderBookSnapshot, Symbol,
    TopOfBookEvent,
};

/// 按 symbol 索引的内存盘口；由 recorder 在写入 WAL 前应用 depth / top 事件。
#[derive(Default)]
pub struct OrderBookStore {
    inner: Arc<RwLock<HashMap<Symbol, SymbolBook>>>,
}

#[derive(Debug, Default, Clone)]
struct SymbolBook {
    bids: Vec<BookLevel>,
    asks: Vec<BookLevel>,
}

impl OrderBookStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn apply_event(&self, event: &MarketEvent) {
        match event {
            MarketEvent::Depth(depth) => {
                let mut books = self.inner.write().await;
                let book = books.entry(depth.symbol.clone()).or_default();
                apply_depth(book, depth);
            }
            MarketEvent::TopOfBook(top) => {
                let mut books = self.inner.write().await;
                let book = books.entry(top.symbol.clone()).or_default();
                apply_top(book, top);
            }
            _ => {}
        }
    }

    pub async fn snapshot_all(&self) -> Vec<OrderBookSnapshot> {
        let books = self.inner.read().await;
        let mut snapshots: Vec<OrderBookSnapshot> = books
            .iter()
            .map(|(symbol, book)| OrderBookSnapshot {
                ts_ns: crate::core::model::now_ns(),
                symbol: symbol.clone(),
                bids: book.bids.clone(),
                asks: book.asks.clone(),
            })
            .collect();
        snapshots.sort_by(|a, b| a.symbol.code.cmp(&b.symbol.code));
        snapshots
    }
}

fn apply_depth(book: &mut SymbolBook, event: &DepthEvent) {
    let levels = match event.side {
        BookSide::Bid => &mut book.bids,
        BookSide::Ask => &mut book.asks,
    };

    let position = event.position as usize;
    let level = BookLevel {
        price: event.price,
        size: event.size,
        market_maker: event.market_maker.clone(),
    };

    match event.operation {
        DepthOperation::Insert => {
            if position <= levels.len() {
                levels.insert(position, level);
            }
        }
        DepthOperation::Update => {
            if let Some(existing) = levels.get_mut(position) {
                *existing = level;
            }
        }
        DepthOperation::Delete => {
            if position < levels.len() {
                levels.remove(position);
            }
        }
    }
}

fn apply_top(book: &mut SymbolBook, event: &TopOfBookEvent) {
    if let Some(bid_px) = event.bid {
        book.bids = vec![BookLevel {
            price: bid_px,
            size: rust_decimal::Decimal::ZERO,
            market_maker: None,
        }];
    }
    if let Some(ask_px) = event.ask {
        book.asks = vec![BookLevel {
            price: ask_px,
            size: rust_decimal::Decimal::ZERO,
            market_maker: None,
        }];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::SecType;
    use rust_decimal::Decimal;

    fn sample_symbol() -> Symbol {
        Symbol {
            code: "00700".into(),
            exchange: "SEHK".into(),
            currency: "HKD".into(),
            sec_type: SecType::Stk,
        }
    }

    fn depth(op: DepthOperation, position: u32, side: BookSide, price: f64) -> DepthEvent {
        DepthEvent {
            ts_recv_ns: 1,
            req_id: 1,
            symbol: sample_symbol(),
            position,
            side,
            operation: op,
            price,
            size: Decimal::ONE,
            market_maker: None,
            is_smart_depth: false,
        }
    }

    #[tokio::test]
    async fn depth_insert_and_snapshot() {
        let store = OrderBookStore::new();
        store
            .apply_event(&MarketEvent::Depth(depth(
                DepthOperation::Insert,
                0,
                BookSide::Bid,
                100.0,
            )))
            .await;
        store
            .apply_event(&MarketEvent::Depth(depth(
                DepthOperation::Insert,
                0,
                BookSide::Ask,
                101.0,
            )))
            .await;

        let snapshots = store.snapshot_all().await;
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].bids.len(), 1);
        assert_eq!(snapshots[0].asks.len(), 1);
    }
}
