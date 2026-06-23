use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::domain::Symbol;

#[derive(Default)]
pub struct SymbolRegistry {
    inner: RwLock<HashMap<i32, Symbol>>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, req_id: i32, symbol: Symbol) {
        self.inner
            .write()
            .expect("symbol registry lock")
            .insert(req_id, symbol);
    }

    pub fn resolve(&self, req_id: i32) -> Option<Symbol> {
        self.inner
            .read()
            .expect("symbol registry lock")
            .get(&req_id)
            .cloned()
    }
}
