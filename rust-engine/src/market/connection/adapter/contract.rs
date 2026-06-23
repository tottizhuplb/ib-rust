use ibapi::contracts::{Contract, SecurityType};

use crate::core::model::{SecType, Symbol};

pub fn equity_contract(symbol: &Symbol) -> Contract {
    Contract {
        symbol: symbol.code.clone().into(),
        exchange: symbol.exchange.clone().into(),
        currency: symbol.currency.clone().into(),
        security_type: match symbol.sec_type {
            SecType::Stk => SecurityType::Stock,
            SecType::Fut => SecurityType::Future,
            SecType::Opt => SecurityType::Option,
            SecType::Ind => SecurityType::Index,
            SecType::Other => SecurityType::Stock,
        },
        ..Default::default()
    }
}
