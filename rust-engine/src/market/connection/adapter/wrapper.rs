use std::sync::Arc;

use super::decoder::SymbolRegistry;
use crate::core::model::{
    now_ns, ApiErrorEvent, ConnectionEvent, ControlEvent, DepthEvent, MarketEvent, TopOfBookEvent,
};
use crate::core::pipeline::EventProducer;

/// 近似无状态的桥接：IB 回调 / 订阅项 → 领域事件。
pub struct IbEventBridge {
    symbols: Arc<SymbolRegistry>,
}

impl IbEventBridge {
    pub fn new(symbols: Arc<SymbolRegistry>) -> Self {
        Self { symbols }
    }

    pub fn publish_connection(&self, events: &mut EventProducer, event: ConnectionEvent) {
        let _ = events.try_publish(MarketEvent::Connection(event));
    }

    pub fn publish_control(&self, events: &mut EventProducer, message: impl Into<String>) {
        let _ = events.try_publish(MarketEvent::Control(ControlEvent {
            ts_ns: now_ns(),
            message: message.into(),
        }));
    }

    pub fn publish_top(&self, events: &mut EventProducer, event: TopOfBookEvent) {
        let _ = events.try_publish(MarketEvent::TopOfBook(event));
    }

    pub fn publish_depth(&self, events: &mut EventProducer, event: DepthEvent) {
        let _ = events.try_publish(MarketEvent::Depth(event));
    }

    pub fn publish_api_error(
        &self,
        events: &mut EventProducer,
        req_id: i32,
        code: i32,
        message: impl Into<String>,
    ) {
        let _ = events.try_publish(MarketEvent::ApiError(ApiErrorEvent {
            ts_ns: now_ns(),
            req_id,
            code,
            message: message.into(),
        }));
    }

    pub fn symbols(&self) -> Arc<SymbolRegistry> {
        Arc::clone(&self.symbols)
    }
}
