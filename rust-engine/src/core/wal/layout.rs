use std::path::{Path, PathBuf};

pub fn checkpoint_path(data_dir: &Path) -> PathBuf {
    data_dir.join("wal.meta")
}

pub fn wal_path(data_dir: &Path) -> PathBuf {
    data_dir.join("wal.jsonl")
}
