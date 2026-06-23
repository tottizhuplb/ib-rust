use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use zstd::stream::write::Encoder;

use super::{layout, segment, wal};
use crate::market::config::StorageConfig;
use crate::core::model::MarketEvent;
use crate::core::pipeline::EventRecorder;

/// 单写者 JSONL 记录器，每段文件 zstd 压缩。
pub struct JsonlZstdRecorder {
    config: StorageConfig,
    segment: segment::Segment,
    wal: wal::WriteAheadLog,
    encoder: Option<Encoder<'static, File>>,
    current_path: PathBuf,
}

impl JsonlZstdRecorder {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        layout::ensure_data_dir(&config)?;
        let segment = segment::Segment::new(1);
        let current_path = layout::segment_path(&config.data_dir, segment.id);
        let encoder = Self::open_encoder(&current_path)?;

        Ok(Self {
            config,
            segment,
            wal: wal::WriteAheadLog::new(),
            encoder: Some(encoder),
            current_path,
        })
    }

    fn open_encoder(path: &PathBuf) -> anyhow::Result<Encoder<'static, File>> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Encoder::new(file, 3)?)
    }

    fn rotate(&mut self) -> anyhow::Result<()> {
        if let Some(mut encoder) = self.encoder.take() {
            encoder.flush()?;
            encoder.finish()?;
        }

        self.wal.mark_flushed(self.segment.id);
        self.segment.id += 1;
        self.segment.bytes_written = 0;
        self.current_path = layout::segment_path(&self.config.data_dir, self.segment.id);
        self.encoder = Some(Self::open_encoder(&self.current_path)?);
        tracing::info!(
            segment = self.segment.id,
            path = %self.current_path.display(),
            "rotated event segment"
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl EventRecorder for JsonlZstdRecorder {
    async fn append(&mut self, event: &MarketEvent) -> anyhow::Result<()> {
        let line = serde_json::to_vec(event)?;
        let encoder = self.encoder.as_mut().expect("segment encoder must be open");

        encoder.write_all(&line)?;
        encoder.write_all(b"\n")?;

        self.segment.bytes_written += (line.len() + 1) as u64;

        if self.segment.bytes_written >= self.config.segment_max_bytes {
            self.rotate()?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> anyhow::Result<()> {
        if let Some(encoder) = self.encoder.as_mut() {
            encoder.flush()?;
        }
        Ok(())
    }

    async fn rotate_if_needed(&mut self) -> anyhow::Result<()> {
        if self.segment.bytes_written >= self.config.segment_max_bytes {
            self.rotate()?;
        }
        Ok(())
    }
}

impl Drop for JsonlZstdRecorder {
    fn drop(&mut self) {
        if let Some(mut encoder) = self.encoder.take() {
            let _ = encoder.flush();
            let _ = encoder.finish();
        }
    }
}
