use anyhow::bail;
use serde::{Deserialize, Deserializer, Serialize};

use crate::core::model::{SecType, Symbol};

/// IB `reqTickByTickData` 的 tickType 请求参数（配置侧，非 response）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TickByTickType {
    Last,
    AllLast,
    BidAsk,
    MidPoint,
}

impl Default for TickByTickType {
    fn default() -> Self {
        Self::Last
    }
}

/// IB 行情 API，与订阅计费项一一对应。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionKind {
    #[serde(rename = "reqMktData")]
    ReqMktData,
    #[serde(rename = "reqTickByTickData")]
    ReqTickByTickData,
    #[serde(rename = "reqMktDepth")]
    ReqMktDepth,
}

impl SubscriptionKind {
    fn parse(name: &str) -> anyhow::Result<Self> {
        match name {
            "reqMktData" => Ok(Self::ReqMktData),
            "reqTickByTickData" => Ok(Self::ReqTickByTickData),
            "reqMktDepth" => Ok(Self::ReqMktDepth),
            other => bail!("unknown subscription mode: {other}"),
        }
    }
}

/// 从配置加载的静态目标集 — 「我们想要什么」。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DesiredSubscription {
    pub symbol: Symbol,
    pub kind: SubscriptionKind,
    /// `reqMktDepth` 档位数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<usize>,
    /// `reqTickByTickData` 的 tickType；默认 Last。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_type: Option<TickByTickType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Pending,
    Active,
    Failed,
    Cancelled,
}

/// 运行时状态 — 「IB 实际给了我们什么」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSubscription {
    pub req_id: i32,
    pub symbol: Symbol,
    pub kind: SubscriptionKind,
    pub tick_type: Option<TickByTickType>,
    pub status: SubscriptionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionKey {
    pub symbol: Symbol,
    pub kind: SubscriptionKind,
    pub tick_type: Option<TickByTickType>,
}

impl DesiredSubscription {
    pub fn key(&self) -> SubscriptionKey {
        SubscriptionKey {
            symbol: self.symbol.clone(),
            kind: self.kind,
            tick_type: self.tick_type,
        }
    }
}

impl ActiveSubscription {
    pub fn key(&self) -> SubscriptionKey {
        SubscriptionKey {
            symbol: self.symbol.clone(),
            kind: self.kind,
            tick_type: self.tick_type,
        }
    }
}

/// conf/market/subscriptions.yaml 中的一行。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionEntry {
    pub symbol: String,
    pub exchange: String,
    /// 单个或 `|` 组合，如 `reqMktData|reqMktDepth`
    #[serde(deserialize_with = "deserialize_modes")]
    pub mode: Vec<SubscriptionKind>,
    /// reqMktDepth 档位数；省略时 runtime 默认 10
    #[serde(default)]
    pub levels: Option<usize>,
    /// reqTickByTickData tickType；省略时默认 Last
    #[serde(default)]
    pub tick_type: Option<TickByTickType>,
}

fn deserialize_modes<'de, D>(deserializer: D) -> Result<Vec<SubscriptionKind>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    parse_modes(&raw).map_err(serde::de::Error::custom)
}

fn parse_modes(raw: &str) -> anyhow::Result<Vec<SubscriptionKind>> {
    let mut kinds = Vec::new();
    for part in raw.split('|').map(str::trim).filter(|part| !part.is_empty()) {
        let kind = SubscriptionKind::parse(part)?;
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    if kinds.is_empty() {
        bail!("mode must list at least one API");
    }
    Ok(kinds)
}

impl SubscriptionEntry {
    pub fn expand_desired(self) -> anyhow::Result<Vec<DesiredSubscription>> {
        let symbol = resolve_symbol(&self)?;
        Ok(self
            .mode
            .into_iter()
            .map(|kind| DesiredSubscription {
                symbol: symbol.clone(),
                kind,
                levels: if kind == SubscriptionKind::ReqMktDepth {
                    self.levels
                } else {
                    None
                },
                tick_type: if kind == SubscriptionKind::ReqTickByTickData {
                    Some(self.tick_type.unwrap_or_default())
                } else {
                    None
                },
            })
            .collect())
    }
}

fn resolve_symbol(entry: &SubscriptionEntry) -> anyhow::Result<Symbol> {
    Ok(Symbol {
        code: entry.symbol.clone(),
        exchange: entry.exchange.clone(),
        currency: infer_currency(&entry.exchange)?,
        sec_type: SecType::Stk,
    })
}

fn infer_currency(exchange: &str) -> anyhow::Result<String> {
    match exchange.to_ascii_uppercase().as_str() {
        "SEHK" | "HKFE" => Ok("HKD".into()),
        "SMART" | "NYSE" | "NASDAQ" | "ISLAND" | "ARCA" | "BATS" | "IEX" | "AMEX" => Ok("USD".into()),
        other => bail!("exchange {other}: cannot infer currency (add mapping in subscription model)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_exchange_and_mode() {
        let entry: SubscriptionEntry = serde_yaml::from_str(
            r#"
symbol: AAPL
exchange: SMART
mode: reqMktData
"#,
        )
        .unwrap();
        let subs = entry.expand_desired().unwrap();
        assert_eq!(subs[0].symbol.currency, "USD");
        assert_eq!(subs[0].symbol.sec_type, SecType::Stk);
    }

    #[test]
    fn hk_exchange_infers_hkd() {
        let entry: SubscriptionEntry = serde_yaml::from_str(
            r#"
symbol: "00700"
exchange: SEHK
mode: reqMktData
"#,
        )
        .unwrap();
        let subs = entry.expand_desired().unwrap();
        assert_eq!(subs[0].symbol.currency, "HKD");
    }

    #[test]
    fn pipe_modes_expand() {
        let entry: SubscriptionEntry = serde_yaml::from_str(
            r#"
symbol: AAPL
exchange: SMART
mode: reqMktData|reqMktDepth
levels: 5
"#,
        )
        .unwrap();
        let subs = entry.expand_desired().unwrap();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[1].levels, Some(5));
    }

    #[test]
    fn missing_mode_fails_at_parse() {
        let err = serde_yaml::from_str::<SubscriptionEntry>(
            r#"
symbol: AAPL
exchange: SMART
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("mode"));
    }

    #[test]
    fn tick_by_tick_gets_default_last() {
        let entry: SubscriptionEntry = serde_yaml::from_str(
            r#"
symbol: AAPL
exchange: SMART
mode: reqTickByTickData
"#,
        )
        .unwrap();
        let subs = entry.expand_desired().unwrap();
        assert_eq!(subs[0].tick_type, Some(TickByTickType::Last));
    }
}
