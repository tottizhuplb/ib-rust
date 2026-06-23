use ibapi::contracts::tick_types::TickType;
use ibapi::market_data::realtime::{BidAsk, MarketDepth, MarketDepthL2, MarketDepths, MidPoint, TickTypes, Trade};
use rust_decimal::Decimal;

use crate::core::model::{
    now_ns, BookSide, DepthEvent, DepthOperation, MarketEvent, TickByTickEvent, TickByTickType,
    TopOfBookEvent,
};
use tokio::sync::mpsc;

use super::super::publish::{try_publish, PublishError};
use crate::core::model::Symbol;

#[derive(Debug, Default)]
pub struct TopQuoteState {
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub last: Option<f64>,
}

pub fn apply_top_tick(state: &mut TopQuoteState, tick: TickTypes) -> bool {
    match tick {
        TickTypes::Price(price) => match price.tick_type {
            TickType::Bid => {
                state.bid = Some(price.price);
                true
            }
            TickType::Ask => {
                state.ask = Some(price.price);
                true
            }
            TickType::Last => {
                state.last = Some(price.price);
                true
            }
            _ => false,
        },
        TickTypes::PriceSize(ps) => match ps.price_tick_type {
            TickType::Bid => {
                state.bid = Some(ps.price);
                true
            }
            TickType::Ask => {
                state.ask = Some(ps.price);
                true
            }
            TickType::Last => {
                state.last = Some(ps.price);
                true
            }
            _ => false,
        },
        _ => false,
    }
}

pub fn top_of_book_event(req_id: i32, symbol: &Symbol, state: &TopQuoteState) -> TopOfBookEvent {
    TopOfBookEvent {
        ts_recv_ns: now_ns(),
        req_id,
        symbol: symbol.clone(),
        bid: state.bid,
        ask: state.ask,
        last: state.last,
    }
}

pub fn depth_event_from_l1(req_id: i32, symbol: &Symbol, depth: &MarketDepth) -> DepthEvent {
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

pub fn depth_event_from_l2(req_id: i32, symbol: &Symbol, depth: &MarketDepthL2) -> DepthEvent {
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
) -> DepthEvent {
    DepthEvent {
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

pub fn publish_top(
    events: &mpsc::Sender<MarketEvent>,
    req_id: i32,
    symbol: &Symbol,
    state: &TopQuoteState,
) -> Result<(), PublishError> {
    try_publish(
        events,
        MarketEvent::TopOfBook(top_of_book_event(req_id, symbol, state)),
    )
}

pub fn publish_depth(
    events: &mpsc::Sender<MarketEvent>,
    update: MarketDepths,
    req_id: i32,
    symbol: &Symbol,
) -> Result<(), PublishError> {
    let event = match update {
        MarketDepths::MarketDepth(depth) => MarketEvent::Depth(depth_event_from_l1(req_id, symbol, &depth)),
        MarketDepths::MarketDepthL2(depth) => {
            MarketEvent::Depth(depth_event_from_l2(req_id, symbol, &depth))
        }
    };
    try_publish(events, event)
}

pub fn publish_tick_by_tick_trade(
    events: &mpsc::Sender<MarketEvent>,
    req_id: i32,
    symbol: &Symbol,
    tick_type: TickByTickType,
    trade: &Trade,
) -> Result<(), PublishError> {
    try_publish(
        events,
        MarketEvent::TickByTick(TickByTickEvent {
            ts_recv_ns: now_ns(),
            req_id,
            symbol: symbol.clone(),
            tick_type,
            price: trade.price,
            size: trade.size,
            bid: None,
            ask: None,
            bid_size: None,
            ask_size: None,
            exchange: Some(trade.exchange.clone()),
        }),
    )
}

pub fn publish_tick_by_tick_bid_ask(
    events: &mpsc::Sender<MarketEvent>,
    req_id: i32,
    symbol: &Symbol,
    quote: &BidAsk,
) -> Result<(), PublishError> {
    try_publish(
        events,
        MarketEvent::TickByTick(TickByTickEvent {
            ts_recv_ns: now_ns(),
            req_id,
            symbol: symbol.clone(),
            tick_type: TickByTickType::BidAsk,
            price: 0.0,
            size: 0.0,
            bid: Some(quote.bid_price),
            ask: Some(quote.ask_price),
            bid_size: Some(quote.bid_size),
            ask_size: Some(quote.ask_size),
            exchange: None,
        }),
    )
}

pub fn publish_tick_by_tick_midpoint(
    events: &mpsc::Sender<MarketEvent>,
    req_id: i32,
    symbol: &Symbol,
    midpoint: &MidPoint,
) -> Result<(), PublishError> {
    try_publish(
        events,
        MarketEvent::TickByTick(TickByTickEvent {
            ts_recv_ns: now_ns(),
            req_id,
            symbol: symbol.clone(),
            tick_type: TickByTickType::MidPoint,
            price: midpoint.mid_point,
            size: 0.0,
            bid: None,
            ask: None,
            bid_size: None,
            ask_size: None,
            exchange: None,
        }),
    )
}

fn decimal_from_f64(value: f64) -> Decimal {
    Decimal::try_from(value).unwrap_or(Decimal::ZERO)
}
