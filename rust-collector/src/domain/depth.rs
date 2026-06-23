use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookSide {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthOperation {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthEvent {
    pub ts_recv_ns: i64,
    pub req_id: i32,
    pub symbol: Symbol,
    pub position: u32,
    pub side: BookSide,
    pub operation: DepthOperation,
    pub price: f64,
    pub size: Decimal,
    pub market_maker: Option<String>,
    pub is_smart_depth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: f64,
    pub size: Decimal,
    pub market_maker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub ts_ns: i64,
    pub symbol: Symbol,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
}
