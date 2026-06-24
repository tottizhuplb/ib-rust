use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use ibapi::market_data::realtime::{BidAsk, MarketDepths, MidPoint, TickTypes, Trade};
use ibapi::subscriptions::{Subscription, SubscriptionItem};
use tokio::sync::Mutex;
use tracing::warn;

use tokio::sync::mpsc;

use crate::core::model::{MarketEvent, Symbol};
use crate::market::connection::adapter::{
    publish_mkt_data, publish_mkt_depth, publish_tick_by_tick, tick_by_tick_bid_ask,
    tick_by_tick_midpoint, tick_by_tick_trade,
};

struct MktDataSubscription {
    symbol: Symbol,
    subscription: Subscription<TickTypes>,
}

struct MktDepthSubscription {
    symbol: Symbol,
    subscription: Subscription<MarketDepths>,
}

enum TickByTickSubscription {
    Trade(Subscription<Trade>),
    BidAsk(Subscription<BidAsk>),
    MidPoint(Subscription<MidPoint>),
}

struct TickByTickEntry {
    symbol: Symbol,
    stream: TickByTickSubscription,
}

pub struct MarketDataStreams {
    mkt_data: Mutex<HashMap<i32, MktDataSubscription>>,
    mkt_depth: Mutex<HashMap<i32, MktDepthSubscription>>,
    tick_by_tick: Mutex<HashMap<i32, TickByTickEntry>>,
}

impl Default for MarketDataStreams {
    fn default() -> Self {
        Self {
            mkt_data: Mutex::new(HashMap::new()),
            mkt_depth: Mutex::new(HashMap::new()),
            tick_by_tick: Mutex::new(HashMap::new()),
        }
    }
}

impl MarketDataStreams {
    pub async fn insert_mkt_data(
        &self,
        req_id: i32,
        symbol: Symbol,
        subscription: Subscription<TickTypes>,
    ) {
        self.mkt_data.lock().await.insert(
            req_id,
            MktDataSubscription {
                symbol,
                subscription,
            },
        );
    }

    pub async fn insert_mkt_depth(
        &self,
        req_id: i32,
        symbol: Symbol,
        subscription: Subscription<MarketDepths>,
    ) {
        self.mkt_depth.lock().await.insert(
            req_id,
            MktDepthSubscription { symbol, subscription },
        );
    }

    pub async fn insert_tick_by_tick_trade(
        &self,
        req_id: i32,
        symbol: Symbol,
        subscription: Subscription<Trade>,
    ) {
        self.tick_by_tick.lock().await.insert(
            req_id,
            TickByTickEntry {
                symbol,
                stream: TickByTickSubscription::Trade(subscription),
            },
        );
    }

    pub async fn insert_tick_by_tick_bid_ask(
        &self,
        req_id: i32,
        symbol: Symbol,
        subscription: Subscription<BidAsk>,
    ) {
        self.tick_by_tick.lock().await.insert(
            req_id,
            TickByTickEntry {
                symbol,
                stream: TickByTickSubscription::BidAsk(subscription),
            },
        );
    }

    pub async fn insert_tick_by_tick_midpoint(
        &self,
        req_id: i32,
        symbol: Symbol,
        subscription: Subscription<MidPoint>,
    ) {
        self.tick_by_tick.lock().await.insert(
            req_id,
            TickByTickEntry {
                symbol,
                stream: TickByTickSubscription::MidPoint(subscription),
            },
        );
    }

    pub async fn clear(&self) {
        self.mkt_data.lock().await.clear();
        self.mkt_depth.lock().await.clear();
        self.tick_by_tick.lock().await.clear();
    }

    pub async fn active_stream_count(&self) -> usize {
        let mkt_data = self.mkt_data.lock().await.len();
        let mkt_depth = self.mkt_depth.lock().await.len();
        let tick_by_tick = self.tick_by_tick.lock().await.len();
        mkt_data + mkt_depth + tick_by_tick
    }

    pub async fn poll(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        any |= self.poll_mkt_data(events).await?;
        any |= self.poll_mkt_depth(events).await?;
        any |= self.poll_tick_by_tick(events).await?;
        Ok(any)
    }

    async fn poll_mkt_data(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        let mut ended = Vec::new();
        let mut mkt_data = self.mkt_data.lock().await;

        for (req_id, sub) in mkt_data.iter_mut() {
            loop {
                match tokio::time::timeout(Duration::from_millis(0), sub.subscription.next()).await {
                    Ok(Some(Ok(SubscriptionItem::Data(tick)))) => {
                        any = true;
                        if let Err(error) = publish_mkt_data(events, *req_id, &sub.symbol, tick) {
                            warn!(req_id, %error, "event channel full or closed");
                        }
                    }
                    Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => {
                        warn!(req_id, code = notice.code, message = %notice.message, "reqMktData notice");
                    }
                    Ok(Some(Err(error))) => {
                        warn!(req_id, error = %error, "reqMktData subscription error");
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
            mkt_data.remove(&req_id);
        }
        Ok(any)
    }

    async fn poll_mkt_depth(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        let mut ended = Vec::new();
        let mut mkt_depth = self.mkt_depth.lock().await;

        for (req_id, sub) in mkt_depth.iter_mut() {
            loop {
                match tokio::time::timeout(Duration::from_millis(0), sub.subscription.next()).await
                {
                    Ok(Some(Ok(SubscriptionItem::Data(update)))) => {
                        any = true;
                        if let Err(error) = publish_mkt_depth(events, update, *req_id, &sub.symbol) {
                            warn!(req_id, %error, "event channel full or closed");
                        }
                    }
                    Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => {
                        warn!(req_id, code = notice.code, message = %notice.message, "reqMktDepth notice");
                    }
                    Ok(Some(Err(error))) => {
                        warn!(req_id, error = %error, "reqMktDepth subscription error");
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
            mkt_depth.remove(&req_id);
        }
        Ok(any)
    }

    async fn poll_tick_by_tick(&self, events: &mpsc::Sender<MarketEvent>) -> anyhow::Result<bool> {
        let mut any = false;
        let mut ended = Vec::new();
        let mut tick_by_tick = self.tick_by_tick.lock().await;

        for (req_id, entry) in tick_by_tick.iter_mut() {
            loop {
                let next = match &mut entry.stream {
                    TickByTickSubscription::Trade(subscription) => {
                        poll_trade_stream(subscription).await
                    }
                    TickByTickSubscription::BidAsk(subscription) => {
                        poll_bid_ask_stream(subscription).await
                    }
                    TickByTickSubscription::MidPoint(subscription) => {
                        poll_midpoint_stream(subscription).await
                    }
                };

                match next {
                    StreamPoll::Idle => break,
                    StreamPoll::Ended => {
                        ended.push(*req_id);
                        break;
                    }
                    StreamPoll::Trade(trade) => {
                        any = true;
                        if let Err(error) = publish_tick_by_tick(
                            events,
                            *req_id,
                            &entry.symbol,
                            tick_by_tick_trade(trade),
                        ) {
                            warn!(req_id, %error, "event channel full or closed");
                        }
                    }
                    StreamPoll::BidAsk(quote) => {
                        any = true;
                        if let Err(error) = publish_tick_by_tick(
                            events,
                            *req_id,
                            &entry.symbol,
                            tick_by_tick_bid_ask(quote),
                        ) {
                            warn!(req_id, %error, "event channel full or closed");
                        }
                    }
                    StreamPoll::MidPoint(midpoint) => {
                        any = true;
                        if let Err(error) = publish_tick_by_tick(
                            events,
                            *req_id,
                            &entry.symbol,
                            tick_by_tick_midpoint(midpoint),
                        ) {
                            warn!(req_id, %error, "event channel full or closed");
                        }
                    }
                    StreamPoll::Notice { code, message } => {
                        warn!(req_id, code, message = %message, "reqTickByTickData notice");
                    }
                    StreamPoll::Error(error) => {
                        warn!(req_id, error = %error, "reqTickByTickData subscription error");
                        ended.push(*req_id);
                        break;
                    }
                }
            }
        }

        for req_id in ended {
            tick_by_tick.remove(&req_id);
        }
        Ok(any)
    }
}

enum StreamPoll {
    Idle,
    Ended,
    Notice { code: i32, message: String },
    Error(ibapi::Error),
    Trade(Trade),
    BidAsk(BidAsk),
    MidPoint(MidPoint),
}

async fn poll_trade_stream(subscription: &mut Subscription<Trade>) -> StreamPoll {
    match tokio::time::timeout(Duration::from_millis(0), subscription.next()).await {
        Ok(Some(Ok(SubscriptionItem::Data(trade)))) => StreamPoll::Trade(trade),
        Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => StreamPoll::Notice {
            code: notice.code,
            message: notice.message,
        },
        Ok(Some(Err(error))) => StreamPoll::Error(error),
        Ok(None) => StreamPoll::Ended,
        Err(_) => StreamPoll::Idle,
    }
}

async fn poll_bid_ask_stream(subscription: &mut Subscription<BidAsk>) -> StreamPoll {
    match tokio::time::timeout(Duration::from_millis(0), subscription.next()).await {
        Ok(Some(Ok(SubscriptionItem::Data(quote)))) => StreamPoll::BidAsk(quote),
        Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => StreamPoll::Notice {
            code: notice.code,
            message: notice.message,
        },
        Ok(Some(Err(error))) => StreamPoll::Error(error),
        Ok(None) => StreamPoll::Ended,
        Err(_) => StreamPoll::Idle,
    }
}

async fn poll_midpoint_stream(subscription: &mut Subscription<MidPoint>) -> StreamPoll {
    match tokio::time::timeout(Duration::from_millis(0), subscription.next()).await {
        Ok(Some(Ok(SubscriptionItem::Data(midpoint)))) => StreamPoll::MidPoint(midpoint),
        Ok(Some(Ok(SubscriptionItem::Notice(notice)))) => StreamPoll::Notice {
            code: notice.code,
            message: notice.message,
        },
        Ok(Some(Err(error))) => StreamPoll::Error(error),
        Ok(None) => StreamPoll::Ended,
        Err(_) => StreamPoll::Idle,
    }
}
