use std::collections::HashMap;

use crate::market::subscription::{
    ActiveSubscription, DesiredSubscription, SubscriptionKey, SubscriptionStatus,
};

/// 维护 desired / active 订阅集与 req_id 映射。
#[derive(Debug, Default)]
pub struct SubscriptionRegistry {
    desired: HashMap<SubscriptionKey, DesiredSubscription>,
    active: HashMap<SubscriptionKey, ActiveSubscription>,
    req_id_map: HashMap<i32, SubscriptionKey>,
    next_req_id: i32,
}

impl SubscriptionRegistry {
    pub fn new(desired: Vec<DesiredSubscription>) -> Self {
        let desired = desired.into_iter().map(|sub| (sub.key(), sub)).collect();
        Self {
            desired,
            next_req_id: 1,
            ..Default::default()
        }
    }

    pub fn desired(&self) -> impl Iterator<Item = &DesiredSubscription> {
        self.desired.values()
    }

    pub fn desired_cloned(&self) -> Vec<DesiredSubscription> {
        self.desired.values().cloned().collect()
    }

    pub fn allocate_req_id(&mut self) -> i32 {
        let id = self.next_req_id;
        self.next_req_id += 1;
        id
    }

    pub fn begin_pending(&mut self, desired: &DesiredSubscription) {
        let key = desired.key();
        self.active.insert(
            key,
            ActiveSubscription {
                req_id: -1,
                symbol: desired.symbol.clone(),
                kind: desired.kind,
                tick_type: desired.tick_type,
                status: SubscriptionStatus::Pending,
            },
        );
    }

    pub fn confirm_active(&mut self, desired: &DesiredSubscription, req_id: i32) {
        let key = desired.key();
        self.req_id_map.insert(req_id, key.clone());
        if let Some(active) = self.active.get_mut(&key) {
            active.req_id = req_id;
            active.status = SubscriptionStatus::Active;
        }
    }

    #[allow(dead_code)]
    pub fn mark_pending(&mut self, desired: &DesiredSubscription) -> i32 {
        self.begin_pending(desired);
        self.allocate_req_id()
    }

    pub fn mark_active(&mut self, key: &SubscriptionKey) {
        if let Some(active) = self.active.get_mut(key) {
            active.status = SubscriptionStatus::Active;
        }
    }

    pub fn mark_failed(&mut self, key: &SubscriptionKey) {
        if let Some(active) = self.active.get_mut(key) {
            active.status = SubscriptionStatus::Failed;
        }
    }

    pub fn clear_active(&mut self) {
        self.active.clear();
        self.req_id_map.clear();
    }

    pub fn has_active(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn resolve_req_id(&self, req_id: i32) -> Option<&SubscriptionKey> {
        self.req_id_map.get(&req_id)
    }
}
