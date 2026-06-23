use tokio::sync::mpsc;

use crate::domain::MarketEvent;
use crate::pipeline::EventRecorder;
use crate::storage::JsonlZstdRecorder;

pub struct RecorderService;

impl RecorderService {
    pub async fn run(
        mut event_rx: mpsc::Receiver<MarketEvent>,
        mut recorder: JsonlZstdRecorder,
    ) -> anyhow::Result<()> {
        let mut batch = 0usize;

        while let Some(event) = event_rx.recv().await {
            recorder.append(&event).await?;
            batch += 1;

            if batch >= 256 {
                recorder.flush().await?;
                batch = 0;
            }
        }

        recorder.flush().await?;
        Ok(())
    }
}
