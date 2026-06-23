use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use super::checkpoint::WalCheckpoint;
use super::layout;
use super::WalConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalRecordKind {
    Event,
    Snapshot,
}

/// 域无关 WAL 写端：seq 由调用方编码进 JSON 行，本层负责 flush / checkpoint。
pub struct WalWriter {
    data_dir: PathBuf,
    checkpoint: WalCheckpoint,
    file: File,
}

impl WalWriter {
    pub fn new(config: WalConfig) -> anyhow::Result<Self> {
        let data_dir = config.ensure_data_dir()?;
        let checkpoint = WalCheckpoint::load(&data_dir)?;
        let wal_path = layout::wal_path(&data_dir);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)?;

        Ok(Self {
            data_dir,
            checkpoint,
            file,
        })
    }

    pub fn last_event_seq(&self) -> u64 {
        self.checkpoint.last_event_seq
    }

    pub fn next_seq(&self) -> u64 {
        self.checkpoint.next_seq
    }

    pub fn append_line(&mut self, seq: u64, line: &[u8], kind: WalRecordKind) -> anyhow::Result<()> {
        debug_assert_eq!(seq, self.checkpoint.next_seq);

        match kind {
            WalRecordKind::Event => self.checkpoint.last_event_seq = seq,
            WalRecordKind::Snapshot => self.checkpoint.last_snapshot_seq = seq,
        }

        self.file.write_all(line)?;
        self.file.write_all(b"\n")?;
        self.checkpoint.next_seq = seq.saturating_add(1);
        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.file.flush()?;
        self.checkpoint.save(&self.data_dir)?;
        Ok(())
    }
}

impl Drop for WalWriter {
    fn drop(&mut self) {
        let _ = self.file.flush();
        let _ = self.checkpoint.save(&self.data_dir);
    }
}
