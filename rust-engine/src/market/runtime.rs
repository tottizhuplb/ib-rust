use std::sync::Arc;

use tokio::sync::{broadcast, watch, Mutex};
use tracing::info;

use crate::core::pipeline::backpressure::event_channel;
use crate::core::task::TaskGroup;
use crate::market::config::{MarketConfig, IB_GATEWAY_HOST};
use crate::market::MarketPhase;
use crate::market::{
    ConnectionManager, HealthService, IbGatewayClient, OrderBookStore, RecorderService,
    SubscriptionManager,
};

/// market 域 shutdown 句柄（worker 由顶层 [`TaskGroup`] 统一 join）。
pub struct MarketHandles {
    phase_tx: watch::Sender<MarketPhase>,
}

impl MarketHandles {
    pub fn begin_shutdown(self, shutdown_tx: &broadcast::Sender<()>) {
        let _ = self.phase_tx.send(MarketPhase::ShuttingDown);
        let _ = shutdown_tx.send(());
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
        wal_dir = %config.storage.wal_data_dir().display(),
        snapshot_interval_secs = config.pipeline.snapshot_interval_secs,
        desired_subscriptions = config.subscriptions.len(),
        "registering market domain"
    );

    let (mut event_producer, event_consumer) =
        event_channel(config.pipeline.event_channel_capacity);
    let (phase_tx, _phase_rx) = watch::channel(MarketPhase::Starting);

    let ib_client = Arc::new(Mutex::new(IbGatewayClient::new(config.ib.clone())));

    let books = Arc::new(OrderBookStore::new());

    tasks.spawn_named("market-recorder", {
        let books = Arc::clone(&books);
        let shutdown_rx = shutdown_tx.subscribe();
        let wal_config = config.storage.wal_config();
        let flush_interval_ms = config.pipeline.flush_interval_ms;
        let snapshot_interval_secs = config.pipeline.snapshot_interval_secs;
        async move {
            RecorderService::run(
                event_consumer,
                wal_config,
                books,
                shutdown_rx,
                flush_interval_ms,
                snapshot_interval_secs,
            )
            .await
        }
    });

    tasks.spawn_named("market-connection", {
        let client = Arc::clone(&ib_client);
        let shutdown_rx = shutdown_tx.subscribe();
        let phase_tx = phase_tx.clone();
        let initial_backoff = config.pipeline.reconnect_backoff_secs;
        async move {
            ConnectionManager::run_supervisor(
                client,
                &mut event_producer,
                shutdown_rx,
                phase_tx,
                initial_backoff,
            )
            .await
        }
    });

    tasks.spawn_named("market-subscription", {
        let desired = config.subscriptions.clone();
        let client = Arc::clone(&ib_client);
        let shutdown_rx = shutdown_tx.subscribe();
        let phase_rx = phase_tx.subscribe();
        async move {
            SubscriptionManager::new(desired, client)
                .run(phase_rx, shutdown_rx)
                .await
        }
    });

    tasks.spawn_named("market-health", {
        let shutdown_rx = shutdown_tx.subscribe();
        let phase_rx = phase_tx.subscribe();
        async move { HealthService::run(shutdown_rx, phase_rx).await }
    });

    Ok(MarketHandles { phase_tx })
}
