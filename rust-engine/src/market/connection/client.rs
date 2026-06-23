use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use ibapi::prelude::*;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::core::model::{now_ns, ConnectionEvent, ControlEvent, MarketEvent, Symbol};
use crate::market::config::IbConfig;

use super::adapter::equity_contract;
use super::market_streams::MarketDataStreams;
use super::publish::try_publish;

/// IB Gateway 客户端封装；订阅流由 [`MarketDataStreams`] 持有并在 reader loop 中轮询。
pub struct IbGatewayClient {
    config: IbConfig,
    client: Option<Arc<Client>>,
    streams: MarketDataStreams,
}

impl IbGatewayClient {
    pub fn new(config: IbConfig) -> Self {
        Self {
            config,
            client: None,
            streams: MarketDataStreams::default(),
        }
    }

    pub fn client(&self) -> Option<&Arc<Client>> {
        self.client.as_ref()
    }

    pub fn streams(&self) -> &MarketDataStreams {
        &self.streams
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub async fn connect(&mut self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<()> {
        self.connect_with_retry(events).await?;

        if let Some(client) = self.client.as_ref() {
            let accounts = client
                .managed_accounts()
                .await
                .context("managed accounts")?;
            info!(?accounts, "managed accounts");
            let _ = try_publish(
                events,
                MarketEvent::Control(ControlEvent {
                    ts_ns: now_ns(),
                    message: format!("managed accounts: {accounts:?}"),
                }),
            );
        }

        Ok(())
    }

    pub async fn disconnect(&mut self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<()> {
        self.unsubscribe_all().await;
        if self.client.take().is_some() {
            let _ = try_publish(
                events,
                MarketEvent::Connection(ConnectionEvent::Disconnected {
                    reason: "client dropped".into(),
                }),
            );
        }
        Ok(())
    }

    /// 取消全部 top / depth；drop ibapi [`Subscription`] 时向 IB 发送 cancel。
    pub async fn unsubscribe_all(&self) {
        self.streams.clear().await;
    }

    pub async fn poll_market_data(
        &mut self,
        events: &mpsc::Sender<MarketEvent>,
    ) -> anyhow::Result<bool> {
        self.streams.poll(events).await
    }

    async fn connect_with_retry(&mut self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<()> {
        const MAX_ATTEMPTS: u32 = 30;
        const RETRY_DELAY: Duration = Duration::from_secs(2);

        let url = self.config.connection_url();

        for attempt in 1..=MAX_ATTEMPTS {
            match Client::connect(&url, self.config.client_id).await {
                Ok(client) => {
                    self.client = Some(Arc::new(client));
                    let _ = try_publish(
                        events,
                        MarketEvent::Connection(ConnectionEvent::Connected {
                            client_id: self.config.client_id,
                        }),
                    );
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

    pub async fn subscribe_market_data(&self, symbol: Symbol) -> anyhow::Result<i32> {
        let client = self.client.as_ref().context("IB client not connected")?;
        let contract = equity_contract(&symbol);
        let subscription = client.market_data(&contract).subscribe().await?;
        let req_id = subscription
            .request_id()
            .context("IB market data subscription missing request_id")?;
        self.streams
            .insert_top(req_id, symbol, subscription)
            .await;
        Ok(req_id)
    }

    pub async fn subscribe_market_depth(&self, symbol: Symbol, levels: usize) -> anyhow::Result<i32> {
        let client = self.client.as_ref().context("IB client not connected")?;
        let contract = equity_contract(&symbol);
        let rows = i32::try_from(levels).unwrap_or(10).clamp(1, 50);
        let subscription = client
            .market_depth(&contract, rows)
            .smart_depth(SmartDepth::No)
            .subscribe()
            .await?;
        let req_id = subscription
            .request_id()
            .context("IB market depth subscription missing request_id")?;
        self.streams
            .insert_depth(req_id, symbol, subscription)
            .await;
        Ok(req_id)
    }
}
