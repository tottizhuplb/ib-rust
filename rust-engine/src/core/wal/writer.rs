use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use super::checkpoint::WalCheckpoint;
use super::layout;
use super::segment::SegmentIdentity;
use super::time::utc_hour_bucket;
use super::WalConfig;
use super::WalRotation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalRecordKind {
    Event,
    Snapshot,
}

/// 域无关 WAL 写端：seq 由调用方编码进 JSON 行，本层负责 rotation / flush / checkpoint。
pub struct WalWriter {
    config: WalConfig,
    data_dir: PathBuf,
    checkpoint: WalCheckpoint,
    file: Option<File>,
    current_path: PathBuf,
}

impl WalWriter {
    pub fn new(config: WalConfig) -> anyhow::Result<Self> {
        let data_dir = config.ensure_data_dir()?;
        let mut checkpoint = WalCheckpoint::load(&data_dir)?;

        if checkpoint.active_segment.hour_bucket == 0 {
            checkpoint.active_segment = initial_segment(&config);
        }

        let current_path = layout::segment_path(&data_dir, &checkpoint.active_segment);
        let file = Some(open_file(&current_path)?);

        Ok(Self {
            config,
            data_dir,
            checkpoint,
            file,
            current_path,
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

        self.maybe_rotate_hourly()?;

        match kind {
            WalRecordKind::Event => self.checkpoint.last_event_seq = seq,
            WalRecordKind::Snapshot => self.checkpoint.last_snapshot_seq = seq,
        }

        self.write_line(line)?;
        self.checkpoint.next_seq = seq.saturating_add(1);
        self.checkpoint.active_segment_bytes = self.active_bytes();

        if self.checkpoint.active_segment_bytes >= self.config.segment_max_bytes {
            self.rotate_part()?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.maybe_rotate_hourly()?;
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }
        self.checkpoint.active_segment_bytes = self.active_bytes();
        self.checkpoint.save(&self.data_dir)?;
        Ok(())
    }

    fn active_bytes(&self) -> u64 {
        self.checkpoint.active_segment_bytes
    }

    fn write_line(&mut self, line: &[u8]) -> anyhow::Result<()> {
        let file = self.file.as_mut().expect("segment file must be open");
        file.write_all(line)?;
        file.write_all(b"\n")?;
        self.checkpoint.active_segment_bytes += (line.len() + 1) as u64;
        Ok(())
    }

    fn maybe_rotate_hourly(&mut self) -> anyhow::Result<()> {
        if self.config.rotation != WalRotation::Hourly {
            return Ok(());
        }
        let bucket = utc_hour_bucket(SystemTime::now());
        if self.checkpoint.active_segment.hour_bucket != bucket {
            self.rotate_to_segment(SegmentIdentity::new(bucket))?;
        }
        Ok(())
    }

    fn rotate_part(&mut self) -> anyhow::Result<()> {
        let next = self.checkpoint.active_segment.next_part();
        self.rotate_to_segment(next)
    }

    fn rotate_to_segment(&mut self, segment: SegmentIdentity) -> anyhow::Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }
        self.checkpoint.active_segment_bytes = self.active_bytes();
        self.checkpoint.save(&self.data_dir)?;

        self.checkpoint.active_segment = segment;
        self.checkpoint.active_segment_bytes = 0;
        self.current_path = layout::segment_path(&self.data_dir, &segment);
        self.file = Some(open_file(&self.current_path)?);

        tracing::info!(
            domain = self.config.domain,
            path = %self.current_path.display(),
            hour_bucket = segment.hour_bucket,
            part = segment.part,
            "rotated wal segment"
        );
        Ok(())
    }
}

impl Drop for WalWriter {
    fn drop(&mut self) {
        if let Some(mut file) = self.file.take() {
            let _ = file.flush();
        }
        let _ = self.checkpoint.save(&self.data_dir);
    }
}

fn open_file(path: &PathBuf) -> anyhow::Result<File> {
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

fn initial_segment(config: &WalConfig) -> SegmentIdentity {
    match config.rotation {
        WalRotation::Hourly => SegmentIdentity::new(utc_hour_bucket(SystemTime::now())),
        WalRotation::SizeOnly => SegmentIdentity::new(1),
    }
}
