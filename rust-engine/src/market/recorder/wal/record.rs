use serde::{Deserialize, Serialize};

use crate::core::model::{MarketEvent, OrderBookSnapshot};
use crate::core::wal::{WalSeq, WalSnapshotRecord};

/// market 域 WAL 记录：`event` 与 `snapshot` 共用全局单调 `seq`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MarketWalRecord {
    Event {
        seq: u64,
        ts_ns: i64,
        event: MarketEvent,
    },
    Snapshot {
        seq: u64,
        ts_ns: i64,
        as_of_seq: u64,
        books: Vec<OrderBookSnapshot>,
    },
}

impl WalSeq for MarketWalRecord {
    fn wal_seq(&self) -> u64 {
        match self {
            Self::Event { seq, .. } | Self::Snapshot { seq, .. } => *seq,
        }
    }
}

impl WalSnapshotRecord for MarketWalRecord {
    fn is_snapshot(&self) -> bool {
        matches!(self, Self::Snapshot { .. })
    }
}

impl MarketWalRecord {
    pub fn event(event: MarketEvent, seq: u64, ts_ns: i64) -> Self {
        Self::Event { seq, ts_ns, event }
    }

    pub fn snapshot(books: Vec<OrderBookSnapshot>, seq: u64, ts_ns: i64, as_of_seq: u64) -> Self {
        Self::Snapshot {
            seq,
            ts_ns,
            as_of_seq,
            books,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{ConnectionEvent, MarketEvent};

    #[test]
    fn market_wal_record_serializes_with_kind_tag() {
        let record = MarketWalRecord::event(
            MarketEvent::Connection(ConnectionEvent::Connected { client_id: 1 }),
            1,
            100,
        );
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"kind\":\"event\""));
        assert!(json.contains("\"seq\":1"));
    }
}
