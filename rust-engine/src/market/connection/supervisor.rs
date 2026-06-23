use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, watch, Mutex};
use tokio::time;
use tracing::warn;

use super::{client::IbGatewayClient, session::IbSession};
use crate::core::model::{now_ns, ApiErrorEvent, ConnectionEvent, MarketEvent};
use crate::core::pipeline::EventProducer;
use crate::market::MarketPhase;

pub struct ConnectionManager;

impl ConnectionManager {
    pub async fn run_supervisor(
        client: Arc<Mutex<IbGatewayClient>>,
        events: &mut EventProducer,
        mut shutdown_rx: broadcast::Receiver<()>,
        phase_tx: watch::Sender<MarketPhase>,
        initial_backoff_secs: u64,
    ) -> anyhow::Result<()> {
        let mut backoff_secs = initial_backoff_secs.max(1);

        loop {
            let _ = phase_tx.send(MarketPhase::Connecting);

            match IbSession::connect_shared(Arc::clone(&client), events).await {
                Ok(mut session) => {
                    let _ = phase_tx.send(MarketPhase::Connected);
                    backoff_secs = initial_backoff_secs.max(1);

                    if !session.wait_until_ready(events).await? {
                        anyhow::bail!("session failed to reach ready state");
                    }

                    tokio::select! {
                        res = session.run_reader_loop() => {
                            if let Err(error) = res {
                                let _ = events.try_publish(MarketEvent::Connection(
                                    ConnectionEvent::Disconnected {
                                        reason: error.to_string(),
                                    },
                                ));
                                warn!(error = %error, "reader loop ended");
                            }
                        }
                        _ = shutdown_rx.recv() => {
                            session.shutdown(events).await?;
                            return Ok(());
                        }
                    }

                    let _ = phase_tx.send(MarketPhase::Recovering);
                    let _ = events.try_publish(MarketEvent::Connection(ConnectionEvent::Disconnected {
                        reason: "recovering".into(),
                    }));

                    tokio::select! {
                        _ = time::sleep(Duration::from_secs(backoff_secs)) => {}
                        _ = shutdown_rx.recv() => return Ok(()),
                    }

                    backoff_secs = (backoff_secs * 2).min(30);
                }
                Err(error) => {
                    let _ = events.try_publish(MarketEvent::ApiError(ApiErrorEvent {
                        ts_ns: now_ns(),
                        req_id: -1,
                        code: -1,
                        message: format!("connect failed: {error:#}"),
                    }));
                    let _ = phase_tx.send(MarketPhase::Recovering);

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
