use std::time::Duration;

use anyhow::Context;
use ibapi::prelude::*;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::core::model::{now_ns, ConnectionEvent, MarketEvent};
use crate::core::pipeline::{
    EventProducer, MarketDataSource, SubscriptionControl, SubscriptionId,
};
use crate::market::config::IbConfig;

/// IB Gateway 客户端封装；订阅流将桥接为 [`MarketEvent`]。
pub struct IbGatewayClient {
    config: IbConfig,
    client: Option<Client>,
}

impl IbGatewayClient {
    pub fn new(config: IbConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    pub fn client(&self) -> Option<&Client> {
        self.client.as_ref()
    }

    async fn connect_with_retry(&mut self, events: &mut EventProducer) -> anyhow::Result<()> {
        const MAX_ATTEMPTS: u32 = 30;
        const RETRY_DELAY: Duration = Duration::from_secs(2);

        let url = self.config.connection_url();

        for attempt in 1..=MAX_ATTEMPTS {
            match Client::connect(&url, self.config.client_id).await {
                Ok(client) => {
                    self.client = Some(client);
                    let _ = events.try_publish(MarketEvent::Connection(
                        ConnectionEvent::Connected {
                            client_id: self.config.client_id,
                        },
                    ));
                    info!(url = %url, "connected to IB Gateway");
                    return Ok(());
                }
                Err(error) => {
                    warn!(attempt, max_attempts = MAX_ATTEMPTS, %error, "IB connection failed");
                    if attempt == MAX_ATTEMPTS {
                        anyhow::bail!("failed to connect to IB Gateway at {url}: {error}");
                    }
                    sleep(RETRY_DELAY).await;
                }
            }
        }

        unreachable!()
    }
}

#[async_trait::async_trait]
impl MarketDataSource for IbGatewayClient {
    async fn connect(&mut self, events: &mut EventProducer) -> anyhow::Result<()> {
        self.connect_with_retry(events).await?;

        if let Some(client) = self.client.as_ref() {
            let accounts = client
                .managed_accounts()
                .await
                .context("managed accounts")?;
            info!(?accounts, "managed accounts");
            let _ = events.try_publish(MarketEvent::Control(crate::core::model::ControlEvent {
                ts_ns: now_ns(),
                message: format!("managed accounts: {accounts:?}"),
            }));
        }

        Ok(())
    }

    async fn disconnect(&mut self, events: &mut EventProducer) -> anyhow::Result<()> {
        if self.client.take().is_some() {
            let _ = events.try_publish(MarketEvent::Connection(ConnectionEvent::Disconnected {
                reason: "client dropped".into(),
            }));
        }
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

#[async_trait::async_trait]
impl SubscriptionControl for IbGatewayClient {
    async fn subscribe_top(
        &self,
        symbol: crate::core::model::Symbol,
    ) -> anyhow::Result<SubscriptionId> {
        let _client = self.client.as_ref().context("IB client not connected")?;
        let _contract = super::adapter::equity_contract(&symbol);

        // TODO: ibapi v3 行情订阅 → 桥接到 TopOfBookEvent
        tracing::info!(?symbol, "subscribe_top queued");
        Ok(symbol.code.parse().unwrap_or(0))
    }

    async fn unsubscribe_top(&self, id: SubscriptionId) -> anyhow::Result<()> {
        tracing::info!(req_id = id, "unsubscribe_top queued");
        Ok(())
    }

    async fn subscribe_depth(
        &self,
        symbol: crate::core::model::Symbol,
        levels: usize,
    ) -> anyhow::Result<SubscriptionId> {
        let _client = self.client.as_ref().context("IB client not connected")?;
        tracing::info!(?symbol, levels, "subscribe_depth queued");
        Ok(symbol.code.parse().unwrap_or(0))
    }

    async fn unsubscribe_depth(&self, id: SubscriptionId) -> anyhow::Result<()> {
        tracing::info!(req_id = id, "unsubscribe_depth queued");
        Ok(())
    }
}
