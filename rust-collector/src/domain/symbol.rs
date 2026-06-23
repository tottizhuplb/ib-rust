use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SecType {
    Stk,
    Fut,
    Opt,
    Ind,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub code: String,
    pub exchange: String,
    pub currency: String,
    pub sec_type: SecType,
}

impl Symbol {
    pub fn hk_equity(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            exchange: "SEHK".into(),
            currency: "HKD".into(),
            sec_type: SecType::Stk,
        }
    }
}
