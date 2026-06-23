use std::sync::Arc;

use crate::domain::{
    now_ns, ApiErrorEvent, ConnectionEvent, ControlEvent, DepthEvent, MarketEvent, TopOfBookEvent,
};
use crate::ib::decoder::SymbolRegistry;
use crate::pipeline::EventPublisher;

/// Stateless-ish bridge: IB callbacks / subscription items -> domain events.
pub struct IbEventBridge {
    publisher: Arc<dyn EventPublisher>,
    symbols: Arc<SymbolRegistry>,
}

impl IbEventBridge {
    pub fn new(publisher: Arc<dyn EventPublisher>, symbols: Arc<SymbolRegistry>) -> Self {
        Self { publisher, symbols }
    }

    pub fn publish_connection(&self, event: ConnectionEvent) {
        let _ = self.publisher.publish(MarketEvent::Connection(event));
    }

    pub fn publish_control(&self, message: impl Into<String>) {
        let _ = self.publisher.publish(MarketEvent::Control(ControlEvent {
            ts_ns: now_ns(),
            message: message.into(),
        }));
    }

    pub fn publish_top(&self, event: TopOfBookEvent) {
        let _ = self.publisher.publish(MarketEvent::TopOfBook(event));
    }

    pub fn publish_depth(&self, event: DepthEvent) {
        let _ = self.publisher.publish(MarketEvent::Depth(event));
    }

    pub fn publish_api_error(&self, req_id: i32, code: i32, message: impl Into<String>) {
        let _ = self.publisher.publish(MarketEvent::ApiError(ApiErrorEvent {
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
