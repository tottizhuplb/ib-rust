use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::sync::{mpsc, Mutex};
use tokio::time;

use super::client::IbGatewayClient;
use crate::core::model::{now_ns, ConnectionEvent, ControlEvent, MarketEvent};
use super::publish::try_publish;
use crate::market::config::IbConfig;

/// 一次 IB Gateway 会话：连接、保活、断开。
pub struct IbSession {
    client: Arc<Mutex<IbGatewayClient>>,
}

impl IbSession {
    pub fn new(client: Arc<Mutex<IbGatewayClient>>) -> Self {
        Self { client }
    }

    pub async fn connect_shared(
        client: Arc<Mutex<IbGatewayClient>>,
        events: &mpsc::Sender<MarketEvent>,
    ) -> anyhow::Result<Self> {
        {
            let mut guard = client.lock().await;
            guard.connect(events).await.context("IB connect")?;
        }

        Ok(Self { client })
    }

    #[allow(dead_code)]
    pub async fn connect(config: &IbConfig, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<Self> {
        let client = Arc::new(Mutex::new(IbGatewayClient::new(config.clone())));
        Self::connect_shared(client, events).await
    }

    pub async fn wait_until_ready(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let ready = {
            let guard = self.client.lock().await;
            guard.is_connected()
        };

        if ready {
            let _ = try_publish(
                events,
                MarketEvent::Connection(ConnectionEvent::Ready {
                    next_order_id: 0,
                }),
            );
            let _ = try_publish(
                events,
                MarketEvent::Control(ControlEvent {
                    ts_ns: now_ns(),
                    message: "session ready".into(),
                }),
            );
            tracing::info!("IB session ready");
        }

        Ok(ready)
    }

    /// 轮询 IB 行情订阅并写入 event channel；连接丢失时返回 Err。
    pub async fn run_reader_loop(
        client: Arc<Mutex<IbGatewayClient>>,
        events: &mpsc::Sender<MarketEvent>,
    ) -> anyhow::Result<()> {
        loop {
            {
                let mut guard = client.lock().await;
                if !guard.is_connected() {
                    anyhow::bail!("IB connection lost");
                }
                guard.poll_market_data(events).await?;
            }
            time::sleep(Duration::from_millis(1)).await;
        }
    }

    pub async fn shutdown(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<()> {
        let mut guard = self.client.lock().await;
        guard.disconnect(events).await.context("IB disconnect")
    }
}
