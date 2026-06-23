use serde::{Deserialize, Serialize};

use super::{
    connection::{ConnectionEvent, ControlEvent},
    depth::DepthEvent,
    error::ApiErrorEvent,
    top::TopOfBookEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    Connection(ConnectionEvent),
    TopOfBook(TopOfBookEvent),
    Depth(DepthEvent),
    Control(ControlEvent),
    ApiError(ApiErrorEvent),
}

pub fn now_ns() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos() as i64
}
