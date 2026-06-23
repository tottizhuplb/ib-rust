use std::fmt;

use crate::core::domain::MarketEvent;

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

pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: MarketEvent) -> Result<(), PublishError>;
}

pub type SubscriptionId = i32;

#[async_trait::async_trait]
pub trait MarketDataSource: Send {
    async fn connect(&mut self) -> anyhow::Result<()>;
    async fn disconnect(&mut self) -> anyhow::Result<()>;
    async fn is_connected(&self) -> bool;
}

#[async_trait::async_trait]
pub trait SubscriptionControl: Send + Sync {
    async fn subscribe_top(
        &self,
        symbol: crate::core::domain::Symbol,
    ) -> anyhow::Result<SubscriptionId>;
    async fn unsubscribe_top(&self, id: SubscriptionId) -> anyhow::Result<()>;

    async fn subscribe_depth(
        &self,
        symbol: crate::core::domain::Symbol,
        levels: usize,
    ) -> anyhow::Result<SubscriptionId>;

    async fn unsubscribe_depth(&self, id: SubscriptionId) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait EventRecorder: Send {
    async fn append(&mut self, event: &MarketEvent) -> anyhow::Result<()>;
    async fn flush(&mut self) -> anyhow::Result<()>;
    async fn rotate_if_needed(&mut self) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait SnapshotStore: Send + Sync {
    async fn save_book(
        &self,
        symbol: &crate::core::domain::Symbol,
        snapshot: &crate::core::domain::OrderBookSnapshot,
    ) -> anyhow::Result<()>;

    async fn load_book(
        &self,
        symbol: &crate::core::domain::Symbol,
    ) -> anyhow::Result<Option<crate::core::domain::OrderBookSnapshot>>;
}
