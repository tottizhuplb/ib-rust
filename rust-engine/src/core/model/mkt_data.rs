use ibapi::market_data::realtime::{
    TickGeneric, TickPrice, TickPriceSize, TickRequestParameters, TickSize, TickString, TickTypes,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::symbol::Symbol;

/// `reqMktData` 推送：envelope + IB 原始 tick（不合并 bid/ask/last）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MktDataEvent {
    pub ts_recv_ns: i64,
    pub req_id: i32,
    pub symbol: Symbol,
    pub tick: Value,
}

pub fn mkt_data_event(req_id: i32, symbol: &Symbol, tick: TickTypes) -> MktDataEvent {
    MktDataEvent {
        ts_recv_ns: crate::core::model::now_ns(),
        req_id,
        symbol: symbol.clone(),
        tick: encode_tick(tick),
    }
}

fn encode_tick(tick: TickTypes) -> Value {
    match tick {
        TickTypes::Price(p) => price_json("price", &p),
        TickTypes::Size(s) => size_json("size", &s),
        TickTypes::PriceSize(ps) => json!({
            "kind": "priceSize",
            "priceTickType": tick_type_name(&ps.price_tick_type),
            "price": ps.price,
            "attributes": tick_attributes_json(&ps.attributes),
            "sizeTickType": tick_type_name(&ps.size_tick_type),
            "size": ps.size,
        }),
        TickTypes::String(s) => string_json(&s),
        TickTypes::Generic(g) => generic_json(&g),
        TickTypes::OptionComputation(c) => json!({
            "kind": "optionComputation",
            "debug": format!("{c:?}"),
        }),
        TickTypes::SnapshotEnd => json!({ "kind": "snapshotEnd" }),
        TickTypes::RequestParameters(p) => request_parameters_json(&p),
        TickTypes::MarketDataType(t) => json!({
            "kind": "marketDataType",
            "dataType": format!("{t:?}"),
        }),
    }
}

fn price_json(kind: &str, p: &TickPrice) -> Value {
    json!({
        "kind": kind,
        "tickType": tick_type_name(&p.tick_type),
        "price": p.price,
        "attributes": tick_attributes_json(&p.attributes),
    })
}

fn size_json(kind: &str, s: &TickSize) -> Value {
    json!({
        "kind": kind,
        "tickType": tick_type_name(&s.tick_type),
        "size": s.size,
    })
}

fn string_json(s: &TickString) -> Value {
    json!({
        "kind": "string",
        "tickType": tick_type_name(&s.tick_type),
        "value": s.value,
    })
}

fn generic_json(g: &TickGeneric) -> Value {
    json!({
        "kind": "generic",
        "tickType": tick_type_name(&g.tick_type),
        "value": g.value,
    })
}

fn request_parameters_json(p: &TickRequestParameters) -> Value {
    json!({
        "kind": "requestParameters",
        "minTick": p.min_tick,
        "bboExchange": p.bbo_exchange,
        "snapshotPermissions": p.snapshot_permissions,
    })
}

fn tick_attributes_json(a: &ibapi::market_data::realtime::TickAttribute) -> Value {
    json!({
        "canAutoExecute": a.can_auto_execute,
        "pastLimit": a.past_limit,
        "preOpen": a.pre_open,
    })
}

fn tick_type_name(tick_type: &ibapi::contracts::tick_types::TickType) -> String {
    format!("{tick_type:?}")
}
