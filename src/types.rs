use std::collections::BTreeMap;

use rust_decimal::Decimal;

/// Pregão histórico normalizado em `Decimal` obtido do endpoint `v8/finance/chart`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PregaoChart {
    pub timestamp_secs: i64,
    pub close: Decimal,
    pub volume: Decimal,
}

/// Snapshot normalizado de indicadores fundamentalistas obtido do endpoint `v10/finance/quoteSummary`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteSummaryFundamentos {
    pub fonte: String,
    pub data_referencia_fonte: Option<String>,
    pub campos: BTreeMap<String, Decimal>,
}
