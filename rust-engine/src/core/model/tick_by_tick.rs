use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

/// IB `reqTickByTickData` 的 tickType 参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TickByTickType {
    Last,
    AllLast,
    BidAsk,
    MidPoint,
}

impl Default for TickByTickType {
    fn default() -> Self {
        Self::Last
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickByTickEvent {
    pub ts_recv_ns: i64,
    pub req_id: i32,
    pub symbol: Symbol,
    pub tick_type: TickByTickType,
    pub price: f64,
    pub size: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub bid_size: Option<f64>,
    pub ask_size: Option<f64>,
    pub exchange: Option<String>,
}
