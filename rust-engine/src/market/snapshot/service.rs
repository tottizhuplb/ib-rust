use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
use tokio::time::{self, Duration};
use tracing::info;

use crate::market::state::OrderBookStore;
use crate::market::wal::MarketWalWriter;

pub struct SnapshotService;

impl SnapshotService {
    pub async fn run(
        books: Arc<OrderBookStore>,
        wal: Arc<Mutex<MarketWalWriter>>,
        mut shutdown_rx: broadcast::Receiver<()>,
        interval_secs: u64,
    ) -> anyhow::Result<()> {
        let mut ticker = time::interval(Duration::from_secs(interval_secs));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let snapshots = books.snapshot_all().await;
                    if snapshots.is_empty() {
                        continue;
                    }
                    let mut writer = wal.lock().await;
                    let seq = writer.append_snapshot(&snapshots)?;
                    writer.flush()?;
                    info!(
                        wal_seq = seq,
                        as_of_seq = writer.last_event_seq(),
                        books = snapshots.len(),
                        "wal snapshot appended"
                    );
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }
}
