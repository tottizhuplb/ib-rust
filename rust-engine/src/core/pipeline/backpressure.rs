use tokio::sync::mpsc;

use crate::core::model::MarketEvent;

use super::bus::PublishError;

/// 有界 channel 指标钩子；接入 Prometheus 时可在此扩展。
pub struct BackpressureMonitor {
    capacity: usize,
}

impl BackpressureMonitor {
    pub fn new(capacity: usize) -> Self {
        Self { capacity }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn on_publish_full(&self, event: &MarketEvent) {
        tracing::warn!(?event, capacity = self.capacity, "event channel full");
    }
}

/// SPSC 写端：不可 [`Clone`]，全进程仅允许一个实例持有。
pub struct EventProducer {
    tx: mpsc::Sender<MarketEvent>,
}

impl EventProducer {
    pub fn try_publish(&mut self, event: MarketEvent) -> Result<(), PublishError> {
        self.tx.try_send(event).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => PublishError::ChannelFull,
            mpsc::error::TrySendError::Closed(_) => PublishError::ChannelClosed,
        })
    }
}

/// SPSC 读端：不可 [`Clone`]，全进程仅允许一个实例持有。
pub struct EventConsumer {
    rx: mpsc::Receiver<MarketEvent>,
}

impl EventConsumer {
    pub async fn recv(&mut self) -> Option<MarketEvent> {
        self.rx.recv().await
    }

    pub fn try_recv(&mut self) -> Result<MarketEvent, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
}

/// 创建严格 SPSC 行情 event channel；`(EventProducer, EventConsumer)` 均不可 clone。
pub fn event_channel(capacity: usize) -> (EventProducer, EventConsumer) {
    let (tx, rx) = mpsc::channel(capacity);
    (
        EventProducer { tx },
        EventConsumer { rx },
    )
}
