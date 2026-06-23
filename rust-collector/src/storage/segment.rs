#[derive(Debug, Clone)]
pub struct Segment {
    pub id: u64,
    pub bytes_written: u64,
}

impl Segment {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            bytes_written: 0,
        }
    }
}
