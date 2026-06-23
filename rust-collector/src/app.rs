use std::sync::Arc;

use anyhow::Context;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::info;

use crate::config::Config;
use crate::domain::OrderBookSnapshot;
use crate::ib::{decoder::SymbolRegistry, wrapper::IbEventBridge, IbGatewayClient};
use crate::pipeline::{
    backpressure, EventPublisher, EventRecorder, MarketDataSource, MpscPublisher,
    SubscriptionControl,
};
use crate::services::{ConnectionManager, RecorderService};
use crate::storage::JsonlZstdRecorder;

pub struct App {
    config: Config,
    publisher: Arc<dyn EventPublisher>,
    client: Arc<Mutex<IbGatewayClient>>,
    event_rx: Option<mpsc::Receiver<crate::domain::MarketEvent>>,
    _snapshot_rx: Option<mpsc::Receiver<OrderBookSnapshot>>,
}

impl App {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let (event_tx, event_rx) =
            backpressure::event_channel(config.pipeline.event_channel_capacity);
        let (snapshot_tx, snapshot_rx) = mpsc::channel(config.pipeline.snapshot_channel_capacity);

        let publisher = MpscPublisher::new(event_tx);
        let _bridge = IbEventBridge::new(
            Arc::clone(&publisher) as Arc<dyn EventPublisher>,
            Arc::new(SymbolRegistry::new()),
        );
        let _snapshot_tx = snapshot_tx;

        let client = IbGatewayClient::new(config.ib.clone(), Arc::clone(&publisher));
        let _ = JsonlZstdRecorder::new(config.storage.clone())?;

        Ok(Self {
            config,
            publisher,
            client: Arc::new(Mutex::new(client)),
            event_rx: Some(event_rx),
            _snapshot_rx: Some(snapshot_rx),
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        info!(
            host = %self.config.ib.host,
            port = self.config.ib.port,
            client_id = self.config.ib.client_id,
            data_dir = %self.config.storage.data_dir.display(),
            "starting rust-collector"
        );

        let event_rx = self
            .event_rx
            .take()
            .context("event receiver already taken")?;

        let recorder = JsonlZstdRecorder::new(self.config.storage.clone())?;
        let recorder_handle =
            tokio::spawn(async move { RecorderService::run(event_rx, recorder).await });

        let connection_handle = {
            let client = Arc::clone(&self.client);
            tokio::spawn(async move { ConnectionManager::new(client).run().await })
        };

        tokio::signal::ctrl_c().await.context("ctrl-c handler")?;
        info!("shutting down");

        connection_handle.abort();
        drop(self.publisher);

        recorder_handle
            .await
            .context("recorder task join")?
            .context("recorder service")?;

        Ok(())
    }
}

// Keep traits in scope for upcoming tests and service wiring.
#[allow(dead_code)]
trait _AppBounds: MarketDataSource + SubscriptionControl + EventPublisher + EventRecorder {}
