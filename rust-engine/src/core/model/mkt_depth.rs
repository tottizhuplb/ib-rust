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

/// `reqMktDepth` 订阅推送的订单簿增量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MktDepthEvent {
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
