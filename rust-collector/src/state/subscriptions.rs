use std::collections::HashMap;

use crate::domain::{
    ActiveSubscription, DesiredSubscription, SubscriptionKey, SubscriptionStatus,
};

/// Holds desired vs active subscription sets and req_id mappings.
#[derive(Debug, Default)]
pub struct SubscriptionRegistry {
    desired: HashMap<SubscriptionKey, DesiredSubscription>,
    active: HashMap<SubscriptionKey, ActiveSubscription>,
    req_id_map: HashMap<i32, SubscriptionKey>,
    next_req_id: i32,
}

impl SubscriptionRegistry {
    pub fn new(desired: Vec<DesiredSubscription>) -> Self {
        let desired = desired
            .into_iter()
            .map(|sub| (sub.key(), sub))
            .collect();
        Self {
            desired,
            next_req_id: 1,
            ..Default::default()
        }
    }

    pub fn desired(&self) -> impl Iterator<Item = &DesiredSubscription> {
        self.desired.values()
    }

    pub fn active(&self) -> impl Iterator<Item = &ActiveSubscription> {
        self.active.values()
    }

    pub fn allocate_req_id(&mut self) -> i32 {
        let id = self.next_req_id;
        self.next_req_id += 1;
        id
    }

    pub fn mark_pending(&mut self, desired: &DesiredSubscription) -> i32 {
        let req_id = self.allocate_req_id();
        let key = desired.key();
        self.req_id_map.insert(req_id, key.clone());
        self.active.insert(
            key,
            ActiveSubscription {
                req_id,
                symbol: desired.symbol.clone(),
                kind: desired.kind,
                status: SubscriptionStatus::Pending,
            },
        );
        req_id
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

    pub fn keys_to_add(&self) -> Vec<&DesiredSubscription> {
        self.desired
            .iter()
            .filter(|(key, _)| !self.active.contains_key(*key))
            .map(|(_, sub)| sub)
            .collect()
    }

    pub fn keys_to_remove(&self) -> Vec<SubscriptionKey> {
        self.active
            .keys()
            .filter(|key| !self.desired.contains_key(*key))
            .cloned()
            .collect()
    }

    pub fn resolve_req_id(&self, req_id: i32) -> Option<&SubscriptionKey> {
        self.req_id_map.get(&req_id)
    }
}
