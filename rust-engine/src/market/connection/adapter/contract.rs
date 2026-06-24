use ibapi::contracts::{Contract, SecurityType};

use crate::core::model::{SecType, Symbol};

/// IB resolves SEHK equities by numeric symbol without leading zeros (e.g. `700`, not `00700`).
fn ib_symbol_code(symbol: &Symbol) -> String {
    if symbol.exchange.eq_ignore_ascii_case("SEHK")
        && !symbol.code.is_empty()
        && symbol.code.chars().all(|c| c.is_ascii_digit())
    {
        let trimmed = symbol.code.trim_start_matches('0');
        if trimmed.is_empty() {
            symbol.code.clone()
        } else {
            trimmed.to_string()
        }
    } else {
        symbol.code.clone()
    }
}

pub fn equity_contract(symbol: &Symbol) -> Contract {
    Contract {
        symbol: ib_symbol_code(symbol).into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::SecType;

    #[test]
    fn sehk_strips_leading_zeros_for_ib() {
        let symbol = Symbol {
            code: "00700".into(),
            exchange: "SEHK".into(),
            currency: "HKD".into(),
            sec_type: SecType::Stk,
        };
        assert_eq!(equity_contract(&symbol).symbol, "700");
    }

    #[test]
    fn non_sehk_symbol_unchanged() {
        let symbol = Symbol::us_equity("AAPL");
        assert_eq!(equity_contract(&symbol).symbol, "AAPL");
    }
}
