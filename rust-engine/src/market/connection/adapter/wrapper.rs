use std::sync::Arc;

use tokio::sync::mpsc;

use super::decoder::SymbolRegistry;
use crate::core::model::{
    now_ns, ApiErrorEvent, ConnectionEvent, ControlEvent, MarketEvent, MktDataEvent, MktDepthEvent,
};
use super::super::publish::try_publish;

/// 近似无状态的桥接：IB 回调 / 订阅项 → 领域事件。
pub struct IbEventBridge {
    symbols: Arc<SymbolRegistry>,
}

impl IbEventBridge {
    pub fn new(symbols: Arc<SymbolRegistry>) -> Self {
        Self { symbols }
    }

    pub fn publish_connection(&self, events: &mpsc::Sender<MarketEvent>, event: ConnectionEvent) {
        let _ = try_publish(events, MarketEvent::Connection(event));
    }

    pub fn publish_control(&self, events: &mpsc::Sender<MarketEvent>, message: impl Into<String>) {
        let _ = try_publish(
            events,
            MarketEvent::Control(ControlEvent {
                ts_ns: now_ns(),
                message: message.into(),
            }),
        );
    }

    pub fn publish_mkt_data(&self, events: &mpsc::Sender<MarketEvent>, event: MktDataEvent) {
        let _ = try_publish(events, MarketEvent::MktData(event));
    }

    pub fn publish_mkt_depth(&self, events: &mpsc::Sender<MarketEvent>, event: MktDepthEvent) {
        let _ = try_publish(events, MarketEvent::MktDepth(event));
    }

    pub fn publish_api_error(
        &self,
        events: &mpsc::Sender<MarketEvent>,
        req_id: i32,
        code: i32,
        message: impl Into<String>,
    ) {
        let _ = try_publish(
            events,
            MarketEvent::ApiError(ApiErrorEvent {
                ts_ns: now_ns(),
                req_id,
                code,
                message: message.into(),
            }),
        );
    }

    pub fn symbols(&self) -> Arc<SymbolRegistry> {
        Arc::clone(&self.symbols)
    }
}
