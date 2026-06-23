use serde::{Deserialize, Serialize};

use super::{
    connection::{ConnectionEvent, ControlEvent},
    error::ApiErrorEvent,
    mkt_data::MktDataEvent,
    mkt_depth::MktDepthEvent,
    tick_by_tick_data::TickByTickDataEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    Connection(ConnectionEvent),
    Control(ControlEvent),
    ApiError(ApiErrorEvent),
    #[serde(rename = "mktData")]
    MktData(MktDataEvent),
    #[serde(rename = "tickByTickData")]
    TickByTickData(TickByTickDataEvent),
    #[serde(rename = "mktDepth")]
    MktDepth(MktDepthEvent),
}

pub fn now_ns() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos() as i64
}
