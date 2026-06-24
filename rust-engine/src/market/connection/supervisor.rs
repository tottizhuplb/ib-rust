use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, watch, Mutex};
use tokio::time::{self, MissedTickBehavior};
use tracing::{info, warn, Instrument};

use super::{client::IbGatewayClient, session::IbSession};
use crate::core::model::{now_ns, ApiErrorEvent, ConnectionEvent, MarketEvent};
use super::publish::try_publish;
use crate::market::MarketPhase;

pub struct ConnectionManager;

impl ConnectionManager {
    pub async fn run_supervisor(
        client: Arc<Mutex<IbGatewayClient>>,
        events: &mpsc::Sender<MarketEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
        phase_tx: watch::Sender<MarketPhase>,
        initial_backoff_secs: u64,
    ) -> anyhow::Result<()> {
        spawn_status_logger(
            Arc::clone(&client),
            phase_tx.subscribe(),
            shutdown_rx.resubscribe(),
        );

        let mut backoff_secs = initial_backoff_secs.max(1);

        loop {
            let _ = phase_tx.send(MarketPhase::Connecting);

            match IbSession::connect_shared(Arc::clone(&client), events).await {
                Ok(session) => {
                    backoff_secs = initial_backoff_secs.max(1);

                    if !session.wait_until_ready(events).await? {
                        anyhow::bail!("session failed to reach ready state");
                    }

                    let _ = phase_tx.send(MarketPhase::Connected);

                    tokio::select! {
                        res = IbSession::run_reader_loop(Arc::clone(&client), events) => {
                            if let Err(error) = res {
                                let _ = try_publish(
                                    events,
                                    MarketEvent::Connection(ConnectionEvent::Disconnected {
                                        reason: error.to_string(),
                                    }),
                                );
                                warn!(error = %error, "reader loop ended");
                            }
                            if let Err(error) = session.shutdown(events).await {
                                warn!(error = %error, "session shutdown after reader loop ended");
                            }
                        }
                        _ = shutdown_rx.recv() => {
                            session.shutdown(events).await?;
                            return Ok(());
                        }
                    }

                    let _ = phase_tx.send(MarketPhase::Recovering);
                    let _ = try_publish(
                        events,
                        MarketEvent::Connection(ConnectionEvent::Disconnected {
                            reason: "recovering".into(),
                        }),
                    );

                    tokio::select! {
                        _ = time::sleep(Duration::from_secs(backoff_secs)) => {}
                        _ = shutdown_rx.recv() => return Ok(()),
                    }

                    backoff_secs = (backoff_secs * 2).min(30);
                }
                Err(error) => {
                    let _ = try_publish(
                        events,
                        MarketEvent::ApiError(ApiErrorEvent {
                            ts_ns: now_ns(),
                            req_id: -1,
                            code: -1,
                            message: format!("connect failed: {error:#}"),
                        }),
                    );
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

fn spawn_status_logger(
    client: Arc<Mutex<IbGatewayClient>>,
    phase_rx: watch::Receiver<MarketPhase>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    tokio::spawn(
        async move {
            let mut ticker = time::interval(Duration::from_secs(10));
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let phase = phase_rx.borrow().clone();
                        let guard = client.lock().await;
                        let socket_connected = guard.is_connected();
                        let active_streams = guard.active_stream_count().await;
                        info!(
                            phase = ?phase,
                            socket_connected,
                            active_streams,
                            "connection status"
                        );
                    }
                    _ = shutdown_rx.recv() => break,
                }
            }
        }
        .instrument(tracing::info_span!("worker", worker = "market-connection")),
    );
}
