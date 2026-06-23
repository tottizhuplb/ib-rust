use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::layout;
use super::WalConfig;

/// 读取 WAL，用于 recovery 与验证。
pub struct WalReader;

impl WalReader {
    pub fn read_file<T: DeserializeOwned>(path: &Path) -> anyhow::Result<Vec<T>> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        let mut records = Vec::new();
        for line in buf.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }
        Ok(records)
    }

    pub fn read_all<T: DeserializeOwned + WalSeq>(config: &WalConfig) -> anyhow::Result<Vec<T>> {
        let path = layout::wal_path(&config.data_dir());
        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut records = Self::read_file(&path)?;
        records.sort_by_key(WalSeq::wal_seq);
        Ok(records)
    }
}

pub trait WalSeq {
    fn wal_seq(&self) -> u64;
}

pub trait WalSnapshotRecord: WalSeq {
    fn is_snapshot(&self) -> bool;
}

pub fn last_snapshot<T: WalSnapshotRecord>(records: &[T]) -> Option<&T> {
    records.iter().rev().find(|record| record.is_snapshot())
}

pub fn seq_of<T: Serialize>(record: &T) -> anyhow::Result<u64>
where
    T: WalSeq,
{
    Ok(record.wal_seq())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::wal::{WalRecordKind, WalWriter};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestRecord {
        seq: u64,
        kind: String,
        body: String,
    }

    impl WalSeq for TestRecord {
        fn wal_seq(&self) -> u64 {
            self.seq
        }
    }

    fn test_config(dir: &Path) -> WalConfig {
        WalConfig {
            root_dir: dir.to_path_buf(),
            domain: "test",
        }
    }

    #[test]
    fn read_file_roundtrip() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let config = test_config(dir.path());
        let mut writer = WalWriter::new(config.clone())?;

        let record = TestRecord {
            seq: 1,
            kind: "event".into(),
            body: "hello".into(),
        };
        let line = serde_json::to_vec(&record)?;
        writer.append_line(1, &line, WalRecordKind::Event)?;
        writer.flush()?;

        let path = config.wal_path();
        let records: Vec<TestRecord> = WalReader::read_file(&path)?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].seq, 1);
        Ok(())
    }
}
