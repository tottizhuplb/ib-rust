use std::sync::Arc;

use tokio::time::{self, Duration};
use tracing::info;

use crate::core::model::MarketEvent;
use crate::core::pipeline::EventConsumer;
use crate::core::wal::WalConfig;
use super::wal::MarketWalWriter;
use crate::market::state::OrderBookStore;

const WRITER_BATCH_SIZE: usize = 4096;

pub struct RecorderService;

impl RecorderService {
    pub async fn run(
        mut events: EventConsumer,
        wal_config: WalConfig,
        books: Arc<OrderBookStore>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
        flush_interval_ms: u64,
        snapshot_interval_secs: u64,
    ) -> anyhow::Result<()> {
        let mut wal = MarketWalWriter::new(wal_config)?;
        let mut flush_ticker = time::interval(Duration::from_millis(flush_interval_ms));
        let mut snapshot_ticker = time::interval(Duration::from_secs(snapshot_interval_secs));
        let mut batch = Vec::with_capacity(WRITER_BATCH_SIZE);

        loop {
            tokio::select! {
                maybe_event = events.recv() => {
                    match maybe_event {
                        Some(event) => {
                            batch.push(event);
                            if batch.len() >= WRITER_BATCH_SIZE {
                                Self::write_batch(&mut wal, &books, &mut batch).await?;
                            }
                        }
                        None => {
                            Self::write_batch(&mut wal, &books, &mut batch).await?;
                            wal.flush()?;
                            return Ok(());
                        }
                    }
                }
                _ = flush_ticker.tick() => {
                    Self::write_batch(&mut wal, &books, &mut batch).await?;
                    wal.flush()?;
                }
                _ = snapshot_ticker.tick() => {
                    Self::write_snapshot(&mut wal, &books).await?;
                }
                _ = shutdown_rx.recv() => {
                    while let Ok(event) = events.try_recv() {
                        batch.push(event);
                    }
                    Self::write_batch(&mut wal, &books, &mut batch).await?;
                    wal.flush()?;
                    return Ok(());
                }
            }
        }
    }

    async fn write_batch(
        wal: &mut MarketWalWriter,
        books: &Arc<OrderBookStore>,
        batch: &mut Vec<MarketEvent>,
    ) -> anyhow::Result<()> {
        for event in batch.drain(..) {
            books.apply_event(&event).await;
            wal.append_event(&event)?;
        }
        Ok(())
    }

    async fn write_snapshot(
        wal: &mut MarketWalWriter,
        books: &Arc<OrderBookStore>,
    ) -> anyhow::Result<()> {
        let snapshots = books.snapshot_all().await;
        if snapshots.is_empty() {
            return Ok(());
        }
        let seq = wal.append_snapshot(&snapshots)?;
        wal.flush()?;
        info!(
            wal_seq = seq,
            as_of_seq = wal.last_event_seq(),
            books = snapshots.len(),
            "wal snapshot appended"
        );
        Ok(())
    }
}
