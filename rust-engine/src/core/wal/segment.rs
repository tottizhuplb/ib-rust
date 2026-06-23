use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentIdentity {
    /// UTC `YYYYMMDDHH`；[`WalRotation::SizeOnly`] 时为启动序号。
    pub hour_bucket: u64,
    /// 同一小时内因字节上限产生的分片序号，从 1 起。
    pub part: u64,
}

impl SegmentIdentity {
    pub fn new(hour_bucket: u64) -> Self {
        Self {
            hour_bucket,
            part: 1,
        }
    }

    pub fn next_part(&self) -> Self {
        Self {
            hour_bucket: self.hour_bucket,
            part: self.part + 1,
        }
    }
}
