use std::sync::Arc;

use tokio::sync::mpsc;

use crate::domain::MarketEvent;
use crate::pipeline::{EventPublisher, PublishError};

#[derive(Clone)]
pub struct MpscPublisher {
    sender: mpsc::Sender<MarketEvent>,
}

impl MpscPublisher {
    pub fn new(sender: mpsc::Sender<MarketEvent>) -> Arc<Self> {
        Arc::new(Self { sender })
    }
}

impl EventPublisher for MpscPublisher {
    fn publish(&self, event: MarketEvent) -> Result<(), PublishError> {
        self.sender.try_send(event).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => PublishError::ChannelFull,
            mpsc::error::TrySendError::Closed(_) => PublishError::ChannelClosed,
        })
    }
}
