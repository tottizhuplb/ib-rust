use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::WalConfig;

/// 按段读取 WAL，用于 recovery 与验证。
pub struct WalReader;

impl WalReader {
    pub fn read_segment_json(path: &Path) -> anyhow::Result<Vec<serde_json::Value>> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        let mut values = Vec::new();
        for line in buf.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            values.push(serde_json::from_str(&line)?);
        }
        Ok(values)
    }

    pub fn read_segment<T: DeserializeOwned>(path: &Path) -> anyhow::Result<Vec<T>> {
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
        let data_dir = config.data_dir();
        let mut paths: Vec<_> = std::fs::read_dir(&data_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("wal-") && name.ends_with(".jsonl"))
            })
            .collect();
        paths.sort();

        let mut records = Vec::new();
        for path in paths {
            records.extend(Self::read_segment(&path)?);
        }
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
    use crate::core::wal::{WalRecordKind, WalRotation, WalWriter};
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
            segment_max_bytes: 1024 * 1024,
            rotation: WalRotation::Hourly,
        }
    }

    #[test]
    fn read_segment_roundtrip() -> anyhow::Result<()> {
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

        let data_dir = config.data_dir();
        let segment = std::fs::read_dir(&data_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("wal-") && n.ends_with(".jsonl"))
            })
            .expect("segment file");
        let records: Vec<TestRecord> = WalReader::read_segment(&segment)?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].seq, 1);
        Ok(())
    }
}
