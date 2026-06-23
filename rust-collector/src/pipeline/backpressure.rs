use tokio::sync::mpsc;

use crate::domain::MarketEvent;

/// Bounded channel metrics hook; extend when adding Prometheus counters.
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

pub fn event_channel(capacity: usize) -> (mpsc::Sender<MarketEvent>, mpsc::Receiver<MarketEvent>) {
    mpsc::channel(capacity)
}
