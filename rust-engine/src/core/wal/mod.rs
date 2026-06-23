use std::path::PathBuf;

/// WAL 存储配置；各域通过 [`domain`](Self::domain) 使用独立子目录。
#[derive(Debug, Clone)]
pub struct WalConfig {
    /// 根目录，如 `./data`。
    pub root_dir: PathBuf,
    /// 域子目录名，如 `market` → `./data/market`。
    pub domain: &'static str,
}

impl WalConfig {
    pub fn data_dir(&self) -> PathBuf {
        self.root_dir.join(self.domain)
    }

    pub fn ensure_data_dir(&self) -> anyhow::Result<PathBuf> {
        let dir = self.data_dir();
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn checkpoint_path(&self) -> PathBuf {
        layout::checkpoint_path(&self.data_dir())
    }

    pub fn wal_path(&self) -> PathBuf {
        layout::wal_path(&self.data_dir())
    }
}

pub mod checkpoint;
pub mod layout;
pub mod reader;
pub mod writer;

pub use checkpoint::WalCheckpoint;
pub use reader::{last_snapshot, WalReader, WalSeq, WalSnapshotRecord};
pub use writer::{WalRecordKind, WalWriter};
