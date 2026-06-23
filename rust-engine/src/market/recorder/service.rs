use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::core::model::MarketEvent;
use crate::core::wal::WalConfig;
use super::wal::MarketWalWriter;

const WRITER_BATCH_SIZE: usize = 4096;

pub struct RecorderService;

impl RecorderService {
    pub async fn run(
        mut events: mpsc::Receiver<MarketEvent>,
        wal_config: WalConfig,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
        flush_interval_ms: u64,
    ) -> anyhow::Result<()> {
        let mut wal = MarketWalWriter::new(wal_config)?;
        let mut flush_ticker = time::interval(Duration::from_millis(flush_interval_ms));
        let mut batch = Vec::with_capacity(WRITER_BATCH_SIZE);

        loop {
            tokio::select! {
                maybe_event = events.recv() => {
                    match maybe_event {
                        Some(event) => {
                            batch.push(event);
                            if batch.len() >= WRITER_BATCH_SIZE {
                                Self::write_batch(&mut wal, &mut batch)?;
                            }
                        }
                        None => {
                            Self::write_batch(&mut wal, &mut batch)?;
                            wal.flush()?;
                            return Ok(());
                        }
                    }
                }
                _ = flush_ticker.tick() => {
                    Self::write_batch(&mut wal, &mut batch)?;
                    wal.flush()?;
                }
                _ = shutdown_rx.recv() => {
                    while let Ok(event) = events.try_recv() {
                        batch.push(event);
                    }
                    Self::write_batch(&mut wal, &mut batch)?;
                    wal.flush()?;
                    return Ok(());
                }
            }
        }
    }

    fn write_batch(wal: &mut MarketWalWriter, batch: &mut Vec<MarketEvent>) -> anyhow::Result<()> {
        for event in batch.drain(..) {
            wal.append_event(&event)?;
        }
        Ok(())
    }
}
