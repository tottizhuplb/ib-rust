use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorEvent {
    pub ts_ns: i64,
    pub req_id: i32,
    pub code: i32,
    pub message: String,
}
