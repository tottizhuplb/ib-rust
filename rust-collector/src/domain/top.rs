use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopOfBookEvent {
    pub ts_recv_ns: i64,
    pub req_id: i32,
    pub symbol: Symbol,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub last: Option<f64>,
}
