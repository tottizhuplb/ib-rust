use std::fmt;

use tokio::sync::mpsc;

use crate::core::model::MarketEvent;

#[derive(Debug, Clone)]
pub enum PublishError {
    ChannelFull,
    ChannelClosed,
}

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChannelFull => write!(f, "event channel full"),
            Self::ChannelClosed => write!(f, "event channel closed"),
        }
    }
}

impl std::error::Error for PublishError {}

pub fn try_publish(
    tx: &mpsc::Sender<MarketEvent>,
    event: MarketEvent,
) -> Result<(), PublishError> {
    tx.try_send(event).map_err(|error| match error {
        mpsc::error::TrySendError::Full(_) => PublishError::ChannelFull,
        mpsc::error::TrySendError::Closed(_) => PublishError::ChannelClosed,
    })
}
