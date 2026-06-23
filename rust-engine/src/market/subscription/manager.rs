use std::sync::Arc;

use tokio::sync::{broadcast, watch, Mutex};
use tracing::{info, warn};

use crate::core::pipeline::SubscriptionControl;
use crate::market::connection::IbGatewayClient;
use crate::market::subscription::{DesiredSubscription, SubscriptionKind};
use crate::market::MarketPhase;

use super::registry::SubscriptionRegistry;

/// 订阅控制平面：将 desired 对齐为 active。
pub struct SubscriptionManager {
    registry: SubscriptionRegistry,
    client: Arc<Mutex<IbGatewayClient>>,
}

impl SubscriptionManager {
    pub fn new(desired: Vec<DesiredSubscription>, client: Arc<Mutex<IbGatewayClient>>) -> Self {
        Self {
            registry: SubscriptionRegistry::new(desired),
            client,
        }
    }

    pub async fn run(
        mut self,
        mut phase_rx: watch::Receiver<MarketPhase>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                changed = phase_rx.changed() => {
                    changed?;
                    let phase = phase_rx.borrow().clone();
                    self.handle_phase(phase).await?;
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }

    async fn handle_phase(&mut self, phase: MarketPhase) -> anyhow::Result<()> {
        match phase {
            MarketPhase::Connected => self.reconcile().await?,
            MarketPhase::Connecting | MarketPhase::Recovering => {
                if self.registry.has_active() {
                    info!("clearing active subscriptions for reconnect");
                    self.registry.clear_active();
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn reconcile(&mut self) -> anyhow::Result<()> {
        let to_add: Vec<DesiredSubscription> =
            self.registry.keys_to_add().into_iter().cloned().collect();

        for desired in to_add {
            let req_id = self.registry.mark_pending(&desired);
            info!(
                req_id,
                symbol = %desired.symbol.code,
                exchange = %desired.symbol.exchange,
                ?desired.kind,
                levels = ?desired.levels,
                "reconcile: subscribing"
            );

            let result = match desired.kind {
                SubscriptionKind::Top => {
                    let client = self.client.lock().await;
                    client.subscribe_top(desired.symbol.clone()).await
                }
                SubscriptionKind::Depth => {
                    let levels = desired.levels.unwrap_or(10);
                    let client = self.client.lock().await;
                    client.subscribe_depth(desired.symbol.clone(), levels).await
                }
            };

            match result {
                Ok(_actual_req_id) => {
                    self.registry.mark_active(&desired.key());
                    info!(req_id, "subscription active");
                }
                Err(error) => {
                    self.registry.mark_failed(&desired.key());
                    warn!(req_id, error = %error, "subscription failed");
                }
            }
        }

        for key in self.registry.keys_to_remove() {
            warn!(
                symbol = %key.symbol.code,
                ?key.kind,
                "reconcile: would unsubscribe stale active subscription"
            );
        }

        Ok(())
    }
}
