use ibapi::contracts::Contract;

use crate::domain::{SecType, Symbol};

pub fn equity_contract(symbol: &Symbol) -> Contract {
    Contract {
        symbol: symbol.code.clone().into(),
        exchange: symbol.exchange.clone().into(),
        currency: symbol.currency.clone().into(),
        security_type: match symbol.sec_type {
            SecType::Stk => "STK".into(),
            SecType::Fut => "FUT".into(),
            SecType::Opt => "OPT".into(),
            SecType::Ind => "IND".into(),
            SecType::Other => "STK".into(),
        },
        ..Default::default()
    }
}
