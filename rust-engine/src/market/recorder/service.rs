use tokio::sync::broadcast;
use tokio::time::{self, Duration};

use crate::core::model::MarketEvent;
use crate::core::pipeline::EventRecorder;
use crate::market::recorder::JsonlZstdRecorder;

const WRITER_BATCH_SIZE: usize = 4096;

pub struct RecorderService;

impl RecorderService {
    pub async fn run(
        mut event_rx: tokio::sync::mpsc::Receiver<MarketEvent>,
        mut recorder: JsonlZstdRecorder,
        mut shutdown_rx: broadcast::Receiver<()>,
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
                                Self::write_batch(&mut recorder, &mut batch).await?;
                            }
                        }
                        None => {
                            Self::write_batch(&mut recorder, &mut batch).await?;
                            recorder.flush().await?;
                            return Ok(());
                        }
                    }
                }
                _ = ticker.tick() => {
                    Self::write_batch(&mut recorder, &mut batch).await?;
                    recorder.flush().await?;
                }
                _ = shutdown_rx.recv() => {
                    while let Ok(event) = event_rx.try_recv() {
                        batch.push(event);
                    }
                    Self::write_batch(&mut recorder, &mut batch).await?;
                    recorder.flush().await?;
                    return Ok(());
                }
            }
        }
    }

    async fn write_batch(
        recorder: &mut JsonlZstdRecorder,
        batch: &mut Vec<MarketEvent>,
    ) -> anyhow::Result<()> {
        for event in batch.drain(..) {
            recorder.append(&event).await?;
        }
        Ok(())
    }
}
