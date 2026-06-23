use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use ibapi::market_data::realtime::{MarketDepths, TickTypes};
use ibapi::subscriptions::{Subscription, SubscriptionItem};
use tokio::sync::Mutex;
use tracing::warn;

use tokio::sync::mpsc;

use crate::core::model::{MarketEvent, Symbol};
use crate::market::connection::adapter::{
    apply_top_tick, publish_depth, publish_top, TopQuoteState,
};

struct TopSubscription {
    symbol: Symbol,
    quote: TopQuoteState,
    subscription: Subscription<TickTypes>,
}

struct DepthSubscription {
    symbol: Symbol,
    subscription: Subscription<MarketDepths>,
}

pub struct MarketDataStreams {
    top: Mutex<HashMap<i32, TopSubscription>>,
    depth: Mutex<HashMap<i32, DepthSubscription>>,
}

impl Default for MarketDataStreams {
    fn default() -> Self {
        Self {
            top: Mutex::new(HashMap::new()),
            depth: Mutex::new(HashMap::new()),
        }
    }
}

impl MarketDataStreams {
    pub async fn insert_top(&self, req_id: i32, symbol: Symbol, subscription: Subscription<TickTypes>) {
        self.top.lock().await.insert(
            req_id,
            TopSubscription {
                symbol,
                quote: TopQuoteState::default(),
                subscription,
            },
        );
    }

    pub async fn insert_depth(
        &self,
        req_id: i32,
        symbol: Symbol,
        subscription: Subscription<MarketDepths>,
    ) {
        self.depth.lock().await.insert(
            req_id,
            DepthSubscription { symbol, subscription },
        );
    }

    pub async fn clear(&self) {
        self.top.lock().await.clear();
        self.depth.lock().await.clear();
    }

    pub async fn poll(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        any |= self.poll_top(events).await?;
        any |= self.poll_depth(events).await?;
        Ok(any)
    }

    async fn poll_top(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        let mut ended = Vec::new();
        let mut top = self.top.lock().await;

        for (req_id, sub) in top.iter_mut() {
            loop {
                match tokio::time::timeout(Duration::from_millis(0), sub.subscription.next()).await {
                    Ok(Some(Ok(SubscriptionItem::Data(tick)))) => {
                        any = true;
                        if apply_top_tick(&mut sub.quote, tick) {
                            if let Err(error) =
                                publish_top(events, *req_id, &sub.symbol, &sub.quote)
                            {
                                warn!(req_id, %error, "event channel full or closed");
                            }
                        }
                    }
                    Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => {
                        warn!(req_id, code = notice.code, message = %notice.message, "market data notice");
                    }
                    Ok(Some(Err(error))) => {
                        warn!(req_id, error = %error, "top subscription error");
                        ended.push(*req_id);
                        break;
                    }
                    Ok(None) => {
                        ended.push(*req_id);
                        break;
                    }
                    Err(_) => break,
                }
            }
        }

        for req_id in ended {
            top.remove(&req_id);
        }
        Ok(any)
    }

    async fn poll_depth(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        let mut ended = Vec::new();
        let mut depth = self.depth.lock().await;

        for (req_id, sub) in depth.iter_mut() {
            loop {
                match tokio::time::timeout(Duration::from_millis(0), sub.subscription.next()).await
                {
                    Ok(Some(Ok(SubscriptionItem::Data(update)))) => {
                        any = true;
                        if let Err(error) = publish_depth(events, update, *req_id, &sub.symbol) {
                            warn!(req_id, %error, "event channel full or closed");
                        }
                    }
                    Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => {
                        warn!(req_id, code = notice.code, message = %notice.message, "depth notice");
                    }
                    Ok(Some(Err(error))) => {
                        warn!(req_id, error = %error, "depth subscription error");
                        ended.push(*req_id);
                        break;
                    }
                    Ok(None) => {
                        ended.push(*req_id);
                        break;
                    }
                    Err(_) => break,
                }
            }
        }

        for req_id in ended {
            depth.remove(&req_id);
        }
        Ok(any)
    }
}
