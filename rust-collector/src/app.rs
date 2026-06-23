use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, watch, Mutex};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::domain::OrderBookSnapshot;
use crate::ib::IbGatewayClient;
use crate::pipeline::EventPublisher;
use crate::state::RunState;

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run_forever(self) -> anyhow::Result<()> {
        info!(
            host = %self.config.ib.host,
            port = self.config.ib.port,
            client_id = self.config.ib.client_id,
            data_dir = %self.config.storage.data_dir.display(),
            desired_subscriptions = self.config.subscriptions.len(),
            "starting rust-collector"
        );

        let (event_tx, event_rx) = crate::pipeline::backpressure::event_channel(
            self.config.pipeline.event_channel_capacity,
        );
        let (snapshot_tx, snapshot_rx) =
            mpsc::channel::<OrderBookSnapshot>(self.config.pipeline.snapshot_channel_capacity);
        let (shutdown_tx, _) = broadcast::channel::<()>(16);
        let (state_tx, _state_rx) = watch::channel(RunState::Starting);

        let publisher: Arc<dyn EventPublisher> =
            crate::pipeline::MpscPublisher::new(event_tx.clone());

        let ib_client = Arc::new(Mutex::new(IbGatewayClient::new(
            self.config.ib.clone(),
            Arc::clone(&publisher),
        )));

        let books = Arc::new(crate::state::OrderBookStore::new());
        let recorder = crate::storage::JsonlZstdRecorder::new(self.config.storage.clone())?;

        let _snapshot_tx = snapshot_tx;
        let _snapshot_rx = snapshot_rx;

        let mut tasks = tokio::task::JoinSet::new();

        tasks.spawn({
            let shutdown_rx = shutdown_tx.subscribe();
            let flush_interval_ms = self.config.pipeline.flush_interval_ms;
            async move {
                crate::services::RecorderService::run(
                    event_rx,
                    recorder,
                    shutdown_rx,
                    flush_interval_ms,
                )
                .await
            }
        });

        tasks.spawn({
            let books = Arc::clone(&books);
            let storage = self.config.storage.clone();
            let shutdown_rx = shutdown_tx.subscribe();
            let interval_secs = self.config.pipeline.snapshot_interval_secs;
            async move {
                crate::services::SnapshotService::run(books, storage, shutdown_rx, interval_secs)
                    .await
            }
        });

        tasks.spawn({
            let config = self.config.clone();
            let client = Arc::clone(&ib_client);
            let publisher = Arc::clone(&publisher);
            let shutdown_rx = shutdown_tx.subscribe();
            let state_tx = state_tx.clone();
            let initial_backoff = self.config.pipeline.reconnect_backoff_secs;
            async move {
                crate::services::ConnectionManager::run_supervisor(
                    config,
                    client,
                    publisher,
                    shutdown_rx,
                    state_tx,
                    initial_backoff,
                )
                .await
            }
        });

        tasks.spawn({
            let desired = self.config.subscriptions.clone();
            let client = Arc::clone(&ib_client);
            let shutdown_rx = shutdown_tx.subscribe();
            let sub_state_rx = state_tx.subscribe();
            async move {
                crate::services::SubscriptionManager::new(desired, client)
                    .run(sub_state_rx, shutdown_rx)
                    .await
            }
        });

        tasks.spawn({
            let shutdown_rx = shutdown_tx.subscribe();
            let state_rx = state_tx.subscribe();
            async move { crate::services::HealthService::run(shutdown_rx, state_rx).await }
        });

        let shutdown_reason = tokio::select! {
            res = wait_for_shutdown_signal() => {
                res?;
                "signal"
            }
            maybe_task = tasks.join_next() => {
                match maybe_task {
                    Some(Ok(Ok(()))) => "task exited",
                    Some(Ok(Err(e))) => {
                        error!(error = %e, "worker task failed");
                        return Err(e);
                    }
                    Some(Err(e)) => {
                        return Err(anyhow::anyhow!("task join error: {e}"));
                    }
                    None => "all tasks finished",
                }
            }
        };

        info!(reason = shutdown_reason, "initiating graceful shutdown");
        let _ = state_tx.send(RunState::ShuttingDown);
        let _ = shutdown_tx.send(());
        drop(event_tx);

        while let Some(res) = tasks.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => warn!(error = %e, "task stopped with error during shutdown"),
                Err(e) => warn!(error = %e, "task join error during shutdown"),
            }
        }

        info!("rust-collector stopped");
        Ok(())
    }

    pub async fn run(self) -> anyhow::Result<()> {
        self.run_forever().await
    }
}

async fn wait_for_shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

#[allow(dead_code)]
trait _AppBounds:
    crate::pipeline::MarketDataSource
    + crate::pipeline::SubscriptionControl
    + crate::pipeline::EventPublisher
    + crate::pipeline::EventRecorder
{
}
