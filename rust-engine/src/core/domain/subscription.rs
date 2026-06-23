use serde::{Deserialize, Serialize};

use super::symbol::{SecType, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionKind {
    Top,
    Depth,
}

/// 从配置加载的静态目标集 — 「我们想要什么」。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DesiredSubscription {
    pub symbol: Symbol,
    pub kind: SubscriptionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Pending,
    Active,
    Failed,
    Cancelled,
}

/// 运行时状态 — 「IB 实际给了我们什么」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSubscription {
    pub req_id: i32,
    pub symbol: Symbol,
    pub kind: SubscriptionKind,
    pub status: SubscriptionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionKey {
    pub symbol: Symbol,
    pub kind: SubscriptionKind,
}

impl DesiredSubscription {
    pub fn key(&self) -> SubscriptionKey {
        SubscriptionKey {
            symbol: self.symbol.clone(),
            kind: self.kind,
        }
    }
}

impl ActiveSubscription {
    pub fn key(&self) -> SubscriptionKey {
        SubscriptionKey {
            symbol: self.symbol.clone(),
            kind: self.kind,
        }
    }
}

/// yaml 中 `subscriptions:` 下的一行。
#[derive(Debug, Deserialize)]
pub struct SubscriptionEntry {
    pub symbol: String,
    pub exchange: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    pub sec_type: SecType,
    pub mode: SubscriptionKind,
    #[serde(default)]
    pub levels: Option<usize>,
}

fn default_currency() -> String {
    "HKD".into()
}

impl From<SubscriptionEntry> for DesiredSubscription {
    fn from(entry: SubscriptionEntry) -> Self {
        Self {
            symbol: Symbol {
                code: entry.symbol,
                exchange: entry.exchange,
                currency: entry.currency,
                sec_type: entry.sec_type,
            },
            kind: entry.mode,
            levels: entry.levels,
        }
    }
}
