use ibapi::market_data::realtime::{BidAsk, MarketDepth, MarketDepthL2, MarketDepths, MidPoint, TickTypes, Trade};
use rust_decimal::Decimal;

use crate::core::model::{
    mkt_data::mkt_data_event, now_ns, BookSide, DepthOperation, MarketEvent, MktDepthEvent,
    TickByTickData, TickByTickDataEvent,
};
use tokio::sync::mpsc;

use super::super::publish::{try_publish, PublishError};
use crate::core::model::Symbol;

pub fn publish_mkt_data(
    events: &mpsc::Sender<MarketEvent>,
    req_id: i32,
    symbol: &Symbol,
    tick: TickTypes,
) -> Result<(), PublishError> {
    try_publish(
        events,
        MarketEvent::MktData(mkt_data_event(req_id, symbol, tick)),
    )
}

pub fn publish_mkt_depth(
    events: &mpsc::Sender<MarketEvent>,
    update: MarketDepths,
    req_id: i32,
    symbol: &Symbol,
) -> Result<(), PublishError> {
    let event = match update {
        MarketDepths::MarketDepth(depth) => {
            MarketEvent::MktDepth(depth_event_from_l1(req_id, symbol, &depth))
        }
        MarketDepths::MarketDepthL2(depth) => {
            MarketEvent::MktDepth(depth_event_from_l2(req_id, symbol, &depth))
        }
    };
    try_publish(events, event)
}

pub fn publish_tick_by_tick(
    events: &mpsc::Sender<MarketEvent>,
    req_id: i32,
    symbol: &Symbol,
    tick: TickByTickData,
) -> Result<(), PublishError> {
    try_publish(
        events,
        MarketEvent::TickByTickData(TickByTickDataEvent {
            ts_recv_ns: now_ns(),
            req_id,
            symbol: symbol.clone(),
            tick,
        }),
    )
}

pub fn tick_by_tick_trade(trade: Trade) -> TickByTickData {
    TickByTickData::Trade(trade)
}

pub fn tick_by_tick_bid_ask(quote: BidAsk) -> TickByTickData {
    TickByTickData::BidAsk(quote)
}

pub fn tick_by_tick_midpoint(midpoint: MidPoint) -> TickByTickData {
    TickByTickData::MidPoint(midpoint)
}

fn depth_event_from_l1(req_id: i32, symbol: &Symbol, depth: &MarketDepth) -> MktDepthEvent {
    depth_event(
        req_id,
        symbol,
        depth.position,
        depth.side,
        depth.operation,
        depth.price,
        depth.size,
        None,
        false,
    )
}

fn depth_event_from_l2(req_id: i32, symbol: &Symbol, depth: &MarketDepthL2) -> MktDepthEvent {
    depth_event(
        req_id,
        symbol,
        depth.position,
        depth.side,
        depth.operation,
        depth.price,
        depth.size,
        Some(depth.market_maker.clone()),
        depth.smart_depth,
    )
}

fn depth_event(
    req_id: i32,
    symbol: &Symbol,
    position: i32,
    side: i32,
    operation: i32,
    price: f64,
    size: f64,
    market_maker: Option<String>,
    is_smart_depth: bool,
) -> MktDepthEvent {
    MktDepthEvent {
        ts_recv_ns: now_ns(),
        req_id,
        symbol: symbol.clone(),
        position: position.max(0) as u32,
        side: if side == 1 { BookSide::Bid } else { BookSide::Ask },
        operation: match operation {
            0 => DepthOperation::Insert,
            2 => DepthOperation::Delete,
            _ => DepthOperation::Update,
        },
        price,
        size: decimal_from_f64(size),
        market_maker,
        is_smart_depth,
    }
}

fn decimal_from_f64(value: f64) -> Decimal {
    Decimal::try_from(value).unwrap_or(Decimal::ZERO)
}
