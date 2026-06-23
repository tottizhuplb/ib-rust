use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionEvent {
    Connected { client_id: i32 },
    Disconnected { reason: String },
    Ready { next_order_id: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlEvent {
    pub ts_ns: i64,
    pub message: String,
}
