use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::layout;
use super::segment::SegmentIdentity;

/// 落盘 checkpoint，加速 recovery。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalCheckpoint {
    pub next_seq: u64,
    pub last_event_seq: u64,
    pub last_snapshot_seq: u64,
    pub active_segment: SegmentIdentity,
    pub active_segment_bytes: u64,
}

impl Default for WalCheckpoint {
    fn default() -> Self {
        Self {
            next_seq: 1,
            last_event_seq: 0,
            last_snapshot_seq: 0,
            active_segment: SegmentIdentity::new(0),
            active_segment_bytes: 0,
        }
    }
}

impl WalCheckpoint {
    pub fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let path = layout::checkpoint_path(data_dir);
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self, data_dir: &Path) -> anyhow::Result<()> {
        let path = layout::checkpoint_path(data_dir);
        let text = serde_json::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}
