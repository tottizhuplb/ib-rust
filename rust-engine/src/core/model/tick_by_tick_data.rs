use ibapi::market_data::realtime::{BidAsk, MidPoint, Trade};
use serde::{Deserialize, Serialize};

use super::symbol::Symbol;

/// IB `reqTickByTickData` 推送的原始 response（ibapi 解码类型，不再 flatten）。
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TickByTickData {
    Trade(Trade),
    BidAsk(BidAsk),
    MidPoint(MidPoint),
}

impl Clone for TickByTickData {
    fn clone(&self) -> Self {
        match self {
            Self::Trade(t) => Self::Trade(Trade {
                tick_type: t.tick_type.clone(),
                time: t.time,
                price: t.price,
                size: t.size,
                trade_attribute: ibapi::market_data::realtime::TradeAttribute {
                    past_limit: t.trade_attribute.past_limit,
                    unreported: t.trade_attribute.unreported,
                },
                exchange: t.exchange.clone(),
                special_conditions: t.special_conditions.clone(),
            }),
            Self::BidAsk(b) => Self::BidAsk(BidAsk {
                time: b.time,
                bid_price: b.bid_price,
                ask_price: b.ask_price,
                bid_size: b.bid_size,
                ask_size: b.ask_size,
                bid_ask_attribute: ibapi::market_data::realtime::BidAskAttribute {
                    bid_past_low: b.bid_ask_attribute.bid_past_low,
                    ask_past_high: b.bid_ask_attribute.ask_past_high,
                },
            }),
            Self::MidPoint(m) => Self::MidPoint(MidPoint {
                time: m.time,
                mid_point: m.mid_point,
            }),
        }
    }
}

/// 领域 envelope：接收时间、req_id、symbol + IB response body。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TickByTickDataEvent {
    pub ts_recv_ns: i64,
    pub req_id: i32,
    pub symbol: Symbol,
    pub tick: TickByTickData,
}
