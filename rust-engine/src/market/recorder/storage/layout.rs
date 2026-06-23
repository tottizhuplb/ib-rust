use std::path::{Path, PathBuf};

use crate::market::config::StorageConfig;

pub fn ensure_data_dir(config: &StorageConfig) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(&config.data_dir)?;
    Ok(config.data_dir.clone())
}

pub fn segment_path(data_dir: &Path, segment_id: u64) -> PathBuf {
    data_dir.join(format!("events-{segment_id:06}.jsonl.zst"))
}
