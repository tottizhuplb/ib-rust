use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::sync::Mutex;
use tokio::time;
use tracing::info;

use crate::config::IbConfig;
use crate::domain::{now_ns, ConnectionEvent, ControlEvent, MarketEvent};
use crate::ib::IbGatewayClient;
use crate::pipeline::{EventPublisher, MarketDataSource};

/// One IB Gateway session: connect, keep alive, disconnect.
///
/// Callbacks from IB are bridged into [`MarketEvent`] via [`EventPublisher`];
/// this type only owns connection lifecycle and the reader loop.
pub struct IbSession {
    client: Arc<Mutex<IbGatewayClient>>,
    publisher: Arc<dyn EventPublisher>,
}

impl IbSession {
    pub fn new(client: Arc<Mutex<IbGatewayClient>>, publisher: Arc<dyn EventPublisher>) -> Self {
        Self { client, publisher }
    }

    pub async fn connect_shared(
        client: Arc<Mutex<IbGatewayClient>>,
        publisher: Arc<dyn EventPublisher>,
    ) -> anyhow::Result<Self> {
        {
            let mut guard = client.lock().await;
            guard.connect().await.context("IB connect")?;
        }

        Ok(Self { client, publisher })
    }

    #[allow(dead_code)]
    pub async fn connect(
        config: &IbConfig,
        publisher: Arc<dyn EventPublisher>,
    ) -> anyhow::Result<Self> {
        let client = Arc::new(Mutex::new(IbGatewayClient::new(
            config.clone(),
            Arc::clone(&publisher),
        )));
        Self::connect_shared(client, publisher).await
    }

    /// Session ready — Subscription Manager listens on [`RunState`] and reconciles.
    pub async fn wait_until_ready(&mut self) -> anyhow::Result<bool> {
        let ready = {
            let guard = self.client.lock().await;
            guard.is_connected().await
        };

        if ready {
            let _ = self
                .publisher
                .publish(MarketEvent::Connection(ConnectionEvent::Ready {
                    next_order_id: 0,
                }));
            let _ = self.publisher.publish(MarketEvent::Control(ControlEvent {
                ts_ns: now_ns(),
                message: "session ready".into(),
            }));
            info!("IB session ready");
        }

        Ok(ready)
    }

    /// Blocks until the connection is lost or an external error occurs.
    pub async fn run_reader_loop(&mut self) -> anyhow::Result<()> {
        loop {
            time::sleep(Duration::from_secs(1)).await;

            let connected = {
                let guard = self.client.lock().await;
                guard.is_connected().await
            };

            if !connected {
                anyhow::bail!("IB connection lost");
            }
        }
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        let mut guard = self.client.lock().await;
        guard.disconnect().await.context("IB disconnect")
    }

    pub fn client(&self) -> Arc<Mutex<IbGatewayClient>> {
        Arc::clone(&self.client)
    }
}
