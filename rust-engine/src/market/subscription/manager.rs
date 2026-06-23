use std::sync::Arc;

use tokio::sync::{broadcast, watch, Mutex};
use tracing::{info, warn};

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
                info!("unsubscribing all market data streams for reconnect");
                self.client.lock().await.unsubscribe_all().await;
                self.registry.clear_active();
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn reconcile(&mut self) -> anyhow::Result<()> {
        info!("reconcile: unsubscribe all, then subscribe desired");
        self.client.lock().await.unsubscribe_all().await;
        self.registry.clear_active();

        let to_subscribe = self.registry.desired_cloned();

        for desired in to_subscribe {
            self.registry.begin_pending(&desired);
            info!(
                symbol = %desired.symbol.code,
                exchange = %desired.symbol.exchange,
                api = ?desired.kind,
                levels = ?desired.levels,
                tick_type = ?desired.tick_type,
                "reconcile: subscribing"
            );

            let result = match desired.kind {
                SubscriptionKind::ReqMktData => {
                    let client = self.client.lock().await;
                    client.subscribe_req_mkt_data(desired.symbol.clone()).await
                }
                SubscriptionKind::ReqTickByTickData => {
                    let tick_type = desired.tick_type.unwrap_or_default();
                    let client = self.client.lock().await;
                    client
                        .subscribe_req_tick_by_tick(desired.symbol.clone(), tick_type)
                        .await
                }
                SubscriptionKind::ReqMktDepth => {
                    let levels = desired.levels.unwrap_or(10);
                    let client = self.client.lock().await;
                    client
                        .subscribe_req_mkt_depth(desired.symbol.clone(), levels)
                        .await
                }
            };

            match result {
                Ok(req_id) => {
                    self.registry.confirm_active(&desired, req_id);
                    info!(req_id, "subscription active");
                }
                Err(error) => {
                    self.registry.mark_failed(&desired.key());
                    warn!(error = %error, "subscription failed");
                }
            }
        }

        Ok(())
    }
}
