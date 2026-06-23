use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::time::{self, Duration};

use crate::config::StorageConfig;
use crate::market::recorder::JsonlZstdRecorder;
use crate::market::state::OrderBookStore;

pub struct SnapshotService;

impl SnapshotService {
    pub async fn run(
        books: Arc<OrderBookStore>,
        storage: StorageConfig,
        mut shutdown_rx: broadcast::Receiver<()>,
        interval_secs: u64,
    ) -> anyhow::Result<()> {
        let mut ticker = time::interval(Duration::from_secs(interval_secs));
        let _writer = JsonlZstdRecorder::new(storage)?;

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let snapshots = books.snapshot_all().await;
                    if !snapshots.is_empty() {
                        tracing::debug!(count = snapshots.len(), "snapshot tick");
                    }
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }
}
