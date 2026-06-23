use std::path::{Path, PathBuf};

use super::segment::SegmentIdentity;

pub fn checkpoint_path(data_dir: &Path) -> PathBuf {
    data_dir.join("wal.meta")
}

pub fn segment_path(data_dir: &Path, segment: &SegmentIdentity) -> PathBuf {
    let base = format!("wal-{}", segment.hour_bucket);
    let name = if segment.part <= 1 {
        format!("{base}.jsonl")
    } else {
        format!("{base}-{part:03}.jsonl", part = segment.part)
    };
    data_dir.join(name)
}
