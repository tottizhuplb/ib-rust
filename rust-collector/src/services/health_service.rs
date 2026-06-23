use tokio::sync::{broadcast, watch};
use tokio::time::{self, Duration};
use tracing::debug;

use crate::state::RunState;

pub struct HealthService;

impl HealthService {
    pub async fn run(
        mut shutdown_rx: broadcast::Receiver<()>,
        state_rx: watch::Receiver<RunState>,
    ) -> anyhow::Result<()> {
        let mut ticker = time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    debug!(state = ?state_rx.borrow(), "health tick");
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }
}
