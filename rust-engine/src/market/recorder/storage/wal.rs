/// 崩溃恢复元数据占位；后续接入 fsync checkpoint。
#[derive(Debug, Default)]
pub struct WriteAheadLog {
    last_flushed_segment: u64,
}

impl WriteAheadLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_flushed(&mut self, segment_id: u64) {
        self.last_flushed_segment = segment_id;
    }

    pub fn last_flushed_segment(&self) -> u64 {
        self.last_flushed_segment
    }
}
