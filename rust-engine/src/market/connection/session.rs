use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::sync::Mutex;
use tokio::time;
use tracing::info;

use super::client::IbGatewayClient;
use crate::core::model::{now_ns, ConnectionEvent, ControlEvent, MarketEvent};
use crate::core::pipeline::{EventProducer, MarketDataSource};
use crate::market::config::IbConfig;

/// 一次 IB Gateway 会话：连接、保活、断开。
///
/// IB 回调通过 [`EventProducer`] 桥接为 [`MarketEvent`]；
/// 本类型只负责连接生命周期与 reader loop。
pub struct IbSession {
    client: Arc<Mutex<IbGatewayClient>>,
}

impl IbSession {
    pub fn new(client: Arc<Mutex<IbGatewayClient>>) -> Self {
        Self { client }
    }

    pub async fn connect_shared(
        client: Arc<Mutex<IbGatewayClient>>,
        events: &mut EventProducer,
    ) -> anyhow::Result<Self> {
        {
            let mut guard = client.lock().await;
            guard.connect(events).await.context("IB connect")?;
        }

        Ok(Self { client })
    }

    #[allow(dead_code)]
    pub async fn connect(config: &IbConfig, events: &mut EventProducer) -> anyhow::Result<Self> {
        let client = Arc::new(Mutex::new(IbGatewayClient::new(config.clone())));
        Self::connect_shared(client, events).await
    }

    /// 会话就绪 — Subscription Manager 监听 [`MarketPhase`] 并 reconcile。
    pub async fn wait_until_ready(&mut self, events: &mut EventProducer) -> anyhow::Result<bool> {
        let ready = {
            let guard = self.client.lock().await;
            guard.is_connected().await
        };

        if ready {
            let _ = events.try_publish(MarketEvent::Connection(ConnectionEvent::Ready {
                next_order_id: 0,
            }));
            let _ = events.try_publish(MarketEvent::Control(ControlEvent {
                ts_ns: now_ns(),
                message: "session ready".into(),
            }));
            info!("IB session ready");
        }

        Ok(ready)
    }

    /// 阻塞直到连接丢失或发生外部错误。
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

    pub async fn shutdown(&mut self, events: &mut EventProducer) -> anyhow::Result<()> {
        let mut guard = self.client.lock().await;
        guard.disconnect(events).await.context("IB disconnect")
    }

    pub fn client(&self) -> Arc<Mutex<IbGatewayClient>> {
        Arc::clone(&self.client)
    }
}
