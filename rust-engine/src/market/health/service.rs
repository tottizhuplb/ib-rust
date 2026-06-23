use tokio::sync::{broadcast, watch};
use tokio::time::{self, Duration};
use tracing::debug;

use crate::market::MarketPhase;

pub struct HealthService;

impl HealthService {
    pub async fn run(
        mut shutdown_rx: broadcast::Receiver<()>,
        phase_rx: watch::Receiver<MarketPhase>,
    ) -> anyhow::Result<()> {
        let mut ticker = time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    debug!(phase = ?phase_rx.borrow(), "health tick");
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }
}
