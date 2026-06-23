use crate::core::model::{ConnectionEvent, MarketEvent};
use crate::core::wal::WalRotation;
use crate::market::config::StorageConfig;
use crate::market::wal::{MarketWalReader, MarketWalWriter};

#[test]
fn market_wal_uses_domain_subdirectory() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let storage = StorageConfig {
        root_dir: dir.path().to_path_buf(),
        segment_max_bytes: 1024 * 1024,
        wal_rotation: WalRotation::Hourly,
    };

    assert_eq!(
        storage.wal_data_dir(),
        dir.path().join("market")
    );

    let mut writer = MarketWalWriter::new(storage.wal_config())?;
    writer.append_event(&MarketEvent::Connection(ConnectionEvent::Connected {
        client_id: 1,
    }))?;
    writer.flush()?;

    let wal_dir = dir.path().join("market");
    assert!(wal_dir.exists());
    assert!(wal_dir.join("wal.meta").exists());

    let records = MarketWalReader::read_all(&storage.wal_config())?;
    assert_eq!(records.len(), 1);
    Ok(())
}
