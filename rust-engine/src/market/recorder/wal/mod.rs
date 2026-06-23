use crate::core::model::{now_ns, MarketEvent, MktDataEvent};
use crate::core::wal::{WalConfig, WalReader, WalRecordKind, WalWriter};

mod record;
#[cfg(test)]
mod tests;

pub use record::MarketWalRecord;

/// market 域 WAL 写端。
pub struct MarketWalWriter {
    inner: WalWriter,
}

impl MarketWalWriter {
    pub fn new(config: WalConfig) -> anyhow::Result<Self> {
        Ok(Self {
            inner: WalWriter::new(config)?,
        })
    }

    pub fn last_event_seq(&self) -> u64 {
        self.inner.last_event_seq()
    }

    pub fn append_event(&mut self, event: &MarketEvent) -> anyhow::Result<u64> {
        let seq = self.inner.next_seq();
        let record = MarketWalRecord::event(event.clone(), seq, now_ns());
        let line = serde_json::to_vec(&record)?;
        self.inner.append_line(seq, &line, WalRecordKind::Event)?;
        Ok(seq)
    }

    pub fn append_snapshot(&mut self, quotes: &[MktDataEvent]) -> anyhow::Result<u64> {
        let seq = self.inner.next_seq();
        let as_of_seq = self.inner.last_event_seq();
        let record = MarketWalRecord::snapshot(quotes.to_vec(), seq, now_ns(), as_of_seq);
        let line = serde_json::to_vec(&record)?;
        self.inner.append_line(seq, &line, WalRecordKind::Snapshot)?;
        Ok(seq)
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.inner.flush()
    }
}

pub struct MarketWalReader;

impl MarketWalReader {
    pub fn read_all(config: &WalConfig) -> anyhow::Result<Vec<MarketWalRecord>> {
        WalReader::read_all(config)
    }
}
