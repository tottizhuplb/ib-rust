use std::sync::Arc;

use tokio::sync::{broadcast, watch, Mutex};
use tracing::info;

use crate::core::pipeline::EventPublisher;
use crate::core::task::TaskGroup;
use crate::core::RunState;
use crate::market::config::{MarketConfig, IB_GATEWAY_HOST};
use crate::market::{
    ConnectionManager, HealthService, IbGatewayClient, JsonlZstdRecorder, OrderBookStore,
    RecorderService, SnapshotService, SubscriptionManager,
};

/// market 域 shutdown 句柄（worker 由顶层 [`TaskGroup`] 统一 join）。
pub struct MarketHandles {
    state_tx: watch::Sender<RunState>,
    event_tx: tokio::sync::mpsc::Sender<crate::core::model::MarketEvent>,
}

impl MarketHandles {
    pub fn begin_shutdown(self, shutdown_tx: &broadcast::Sender<()>) {
        let _ = self.state_tx.send(RunState::ShuttingDown);
        let _ = shutdown_tx.send(());
        drop(self.event_tx);
    }
}

pub fn register(
    tasks: &mut TaskGroup,
    shutdown_tx: &broadcast::Sender<()>,
    config: MarketConfig,
) -> anyhow::Result<MarketHandles> {
    info!(
        host = IB_GATEWAY_HOST,
        port = config.ib.port,
        client_id = config.ib.client_id,
        data_dir = %config.storage.data_dir.display(),
        desired_subscriptions = config.subscriptions.len(),
        "registering market domain"
    );

    let (event_tx, event_rx) =
        crate::core::pipeline::backpressure::event_channel(config.pipeline.event_channel_capacity);
    let (state_tx, _state_rx) = watch::channel(RunState::Starting);

    let publisher: Arc<dyn EventPublisher> =
        crate::core::pipeline::MpscPublisher::new(event_tx.clone());

    let ib_client = Arc::new(Mutex::new(IbGatewayClient::new(
        config.ib.clone(),
        Arc::clone(&publisher),
    )));

    let books = Arc::new(OrderBookStore::new());
    let recorder = JsonlZstdRecorder::new(config.storage.clone())?;

    tasks.spawn_named("market-recorder", {
        let shutdown_rx = shutdown_tx.subscribe();
        let flush_interval_ms = config.pipeline.flush_interval_ms;
        async move {
            RecorderService::run(event_rx, recorder, shutdown_rx, flush_interval_ms).await
        }
    });

    tasks.spawn_named("market-snapshot", {
        let books = Arc::clone(&books);
        let storage = config.storage.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        let interval_secs = config.pipeline.snapshot_interval_secs;
        async move { SnapshotService::run(books, storage, shutdown_rx, interval_secs).await }
    });

    tasks.spawn_named("market-connection", {
        let client = Arc::clone(&ib_client);
        let publisher = Arc::clone(&publisher);
        let shutdown_rx = shutdown_tx.subscribe();
        let state_tx = state_tx.clone();
        let initial_backoff = config.pipeline.reconnect_backoff_secs;
        async move {
            ConnectionManager::run_supervisor(
                client,
                publisher,
                shutdown_rx,
                state_tx,
                initial_backoff,
            )
            .await
        }
    });

    tasks.spawn_named("market-subscription", {
        let desired = config.subscriptions.clone();
        let client = Arc::clone(&ib_client);
        let shutdown_rx = shutdown_tx.subscribe();
        let sub_state_rx = state_tx.subscribe();
        async move {
            SubscriptionManager::new(desired, client)
                .run(sub_state_rx, shutdown_rx)
                .await
        }
    });

    tasks.spawn_named("market-health", {
        let shutdown_rx = shutdown_tx.subscribe();
        let state_rx = state_tx.subscribe();
        async move { HealthService::run(shutdown_rx, state_rx).await }
    });

    Ok(MarketHandles { state_tx, event_tx })
}
