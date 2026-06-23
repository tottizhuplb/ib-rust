use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, watch, Mutex};
use tokio::time;
use tracing::warn;

use super::{client::IbGatewayClient, session::IbSession};
use crate::core::domain::{now_ns, ApiErrorEvent, ConnectionEvent, MarketEvent};
use crate::core::pipeline::EventPublisher;
use crate::core::RunState;

pub struct ConnectionManager;

impl ConnectionManager {
    pub async fn run_supervisor(
        client: Arc<Mutex<IbGatewayClient>>,
        publisher: Arc<dyn EventPublisher>,
        mut shutdown_rx: broadcast::Receiver<()>,
        state_tx: watch::Sender<RunState>,
        initial_backoff_secs: u64,
    ) -> anyhow::Result<()> {
        let mut backoff_secs = initial_backoff_secs.max(1);

        loop {
            let _ = state_tx.send(RunState::Connecting);

            match IbSession::connect_shared(Arc::clone(&client), Arc::clone(&publisher)).await {
                Ok(mut session) => {
                    let _ = state_tx.send(RunState::Connected);
                    backoff_secs = initial_backoff_secs.max(1);

                    if !session.wait_until_ready().await? {
                        anyhow::bail!("session failed to reach ready state");
                    }

                    tokio::select! {
                        res = session.run_reader_loop() => {
                            if let Err(error) = res {
                                let _ = publisher.publish(MarketEvent::Connection(
                                    ConnectionEvent::Disconnected {
                                        reason: error.to_string(),
                                    },
                                ));
                                warn!(error = %error, "reader loop ended");
                            }
                        }
                        _ = shutdown_rx.recv() => {
                            session.shutdown().await?;
                            return Ok(());
                        }
                    }

                    let _ = state_tx.send(RunState::Recovering);
                    let _ =
                        publisher.publish(MarketEvent::Connection(ConnectionEvent::Disconnected {
                            reason: "recovering".into(),
                        }));

                    tokio::select! {
                        _ = time::sleep(Duration::from_secs(backoff_secs)) => {}
                        _ = shutdown_rx.recv() => return Ok(()),
                    }

                    backoff_secs = (backoff_secs * 2).min(30);
                }
                Err(error) => {
                    let _ = publisher.publish(MarketEvent::ApiError(ApiErrorEvent {
                        ts_ns: now_ns(),
                        req_id: -1,
                        code: -1,
                        message: format!("connect failed: {error:#}"),
                    }));
                    let _ = state_tx.send(RunState::Recovering);

                    tokio::select! {
                        _ = time::sleep(Duration::from_secs(backoff_secs)) => {}
                        _ = shutdown_rx.recv() => return Ok(()),
                    }

                    backoff_secs = (backoff_secs * 2).min(30);
                }
            }
        }
    }
}
