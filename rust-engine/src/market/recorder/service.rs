use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::time::{self, Duration};

use crate::core::model::MarketEvent;
use crate::market::state::OrderBookStore;
use crate::market::wal::MarketWalWriter;

const WRITER_BATCH_SIZE: usize = 4096;

pub struct RecorderService;

impl RecorderService {
    pub async fn run(
        mut event_rx: tokio::sync::mpsc::Receiver<MarketEvent>,
        wal: Arc<Mutex<MarketWalWriter>>,
        books: Arc<OrderBookStore>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
        flush_interval_ms: u64,
    ) -> anyhow::Result<()> {
        let mut ticker = time::interval(Duration::from_millis(flush_interval_ms));
        let mut batch = Vec::with_capacity(WRITER_BATCH_SIZE);

        loop {
            tokio::select! {
                maybe_event = event_rx.recv() => {
                    match maybe_event {
                        Some(event) => {
                            batch.push(event);
                            if batch.len() >= WRITER_BATCH_SIZE {
                                Self::write_batch(&wal, &books, &mut batch).await?;
                            }
                        }
                        None => {
                            Self::write_batch(&wal, &books, &mut batch).await?;
                            wal.lock().await.flush()?;
                            return Ok(());
                        }
                    }
                }
                _ = ticker.tick() => {
                    Self::write_batch(&wal, &books, &mut batch).await?;
                    wal.lock().await.flush()?;
                }
                _ = shutdown_rx.recv() => {
                    while let Ok(event) = event_rx.try_recv() {
                        batch.push(event);
                    }
                    Self::write_batch(&wal, &books, &mut batch).await?;
                    wal.lock().await.flush()?;
                    return Ok(());
                }
            }
        }
    }

    async fn write_batch(
        wal: &Arc<Mutex<MarketWalWriter>>,
        books: &Arc<OrderBookStore>,
        batch: &mut Vec<MarketEvent>,
    ) -> anyhow::Result<()> {
        let mut writer = wal.lock().await;
        for event in batch.drain(..) {
            books.apply_event(&event).await;
            writer.append_event(&event)?;
        }
        Ok(())
    }
}
