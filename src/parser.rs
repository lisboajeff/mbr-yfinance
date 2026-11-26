use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Deserialize;

use crate::{PregaoChart, QuoteSummaryFundamentos, YfinanceError};

#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: YahooChartBody,
}

#[derive(Debug, Deserialize)]
struct YahooChartBody {
    result: Option<Vec<YahooChartResult>>,
}

#[derive(Debug, Deserialize)]
struct YahooChartResult {
    timestamp: Option<Vec<Option<i64>>>,
    indicators: Option<YahooIndicators>,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Option<Vec<YahooQuoteSeries>>,
}

#[derive(Debug, Deserialize)]
struct YahooQuoteSeries {
    close: Option<Vec<Option<f64>>>,
    volume: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
struct YahooNumericField {
    raw: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct YahooDateField {
    fmt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooSummaryDetail {
    #[serde(rename = "trailingPE")]
    trailing_pe: Option<YahooNumericField>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooKeyStatistics {
    price_to_book: Option<YahooNumericField>,
    profit_margins: Option<YahooNumericField>,
    most_recent_quarter: Option<YahooDateField>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooFinancialData {
    return_on_equity: Option<YahooNumericField>,
    profit_margins: Option<YahooNumericField>,
    current_ratio: Option<YahooNumericField>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooQuoteSummaryResult {
    summary_detail: Option<YahooSummaryDetail>,
    default_key_statistics: Option<YahooKeyStatistics>,
    financial_data: Option<YahooFinancialData>,
}

#[derive(Debug, Deserialize)]
struct YahooQuoteSummaryBody {
    result: Option<Vec<YahooQuoteSummaryResult>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooQuoteSummaryResponse {
    quote_summary: YahooQuoteSummaryBody,
}

/// Normaliza um ticker da B3 para o sufixo `.SA` exigido pelo Yahoo Finance.
#[must_use]
pub fn normalizar_ticker_b3(ticker: &str) -> String {
    let upper = ticker.trim().to_ascii_uppercase();
    if upper.ends_with(".SA") {
        upper
    } else {
        format!("{upper}.SA")
    }
}

/// Faz o parsing puro do JSON de `v8/finance/chart` para pregões históricos em `Decimal`.
///
/// # Errors
/// Retorna [`YfinanceError::RespostaInvalida`] em JSON malformado ou [`YfinanceError::AtivoNaoEncontrado`] quando não houver pregões válidos.
pub fn parse_chart_json(ticker: &str, json_str: &str) -> Result<Vec<PregaoChart>, YfinanceError> {
    let payload: YahooChartResponse = serde_json::from_str(json_str)
        .map_err(|e| YfinanceError::RespostaInvalida(e.to_string()))?;

    let result = payload
        .chart
        .result
        .and_then(|mut items| items.drain(..).next())
        .ok_or_else(|| YfinanceError::AtivoNaoEncontrado(ticker.to_string()))?;

    let timestamps = result.timestamp.unwrap_or_default();
    let quote = result
        .indicators
        .and_then(|ind| ind.quote)
        .and_then(|mut q| q.drain(..).next())
        .ok_or_else(|| YfinanceError::AtivoNaoEncontrado(ticker.to_string()))?;

    let closes = quote.close.unwrap_or_default();
    let volumes = quote.volume.unwrap_or_default();

    let len = timestamps.len().min(closes.len());
    let mut pregoes = Vec::with_capacity(len);
    for i in 0..len {
        let Some(ts) = timestamps[i] else {
            continue;
        };
        let Some(close_f64) = closes[i] else {
            continue;
        };
        if !close_f64.is_finite() || close_f64 <= 0.0 {
            continue;
        }
        let Some(close) = Decimal::from_f64(close_f64) else {
            continue;
        };
        let vol_f64 = volumes.get(i).and_then(|v| *v).unwrap_or(0.0);
        let volume = if vol_f64.is_finite() && vol_f64 >= 0.0 {
            Decimal::from_f64(vol_f64).unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        pregoes.push(PregaoChart {
            timestamp_secs: ts,
            close,
            volume,
        });
    }

    if pregoes.is_empty() {
        return Err(YfinanceError::AtivoNaoEncontrado(ticker.to_string()));
    }

    Ok(pregoes)
}

/// Faz o parsing puro do JSON de `v10/finance/quoteSummary` para indicadores em `Decimal`.
///
/// # Errors
/// Retorna [`YfinanceError::RespostaInvalida`] em JSON malformado ou [`YfinanceError::AtivoNaoEncontrado`] quando `result` vier vazio.
pub fn parse_quote_summary_json(
    ticker: &str,
    json_str: &str,
) -> Result<QuoteSummaryFundamentos, YfinanceError> {
    let body: YahooQuoteSummaryResponse = serde_json::from_str(json_str)
        .map_err(|e| YfinanceError::RespostaInvalida(e.to_string()))?;

    let result = body
        .quote_summary
        .result
        .and_then(|mut items| items.drain(..).next())
        .ok_or_else(|| YfinanceError::AtivoNaoEncontrado(ticker.to_string()))?;

    let mut campos = BTreeMap::new();

    if let Some(val) = result
        .summary_detail
        .as_ref()
        .and_then(|s| s.trailing_pe.as_ref())
        .and_then(para_decimal_finito)
    {
        campos.insert("trailingPE".to_string(), val.round_dp(2));
    }

    if let Some(val) = result
        .default_key_statistics
        .as_ref()
        .and_then(|k| k.price_to_book.as_ref())
        .and_then(para_decimal_finito)
    {
        campos.insert("priceToBook".to_string(), val.round_dp(2));
    }

    if let Some(val) = result
        .financial_data
        .as_ref()
        .and_then(|f| f.return_on_equity.as_ref())
        .and_then(para_decimal_finito)
    {
        campos.insert("returnOnEquity".to_string(), val.round_dp(4));
    }

    let profit_margins = result
        .financial_data
        .as_ref()
        .and_then(|f| f.profit_margins.as_ref())
        .and_then(para_decimal_finito)
        .or_else(|| {
            result
                .default_key_statistics
                .as_ref()
                .and_then(|k| k.profit_margins.as_ref())
                .and_then(para_decimal_finito)
        });
    if let Some(val) = profit_margins {
        campos.insert("profitMargins".to_string(), val.round_dp(4));
    }

    if let Some(val) = result
        .financial_data
        .as_ref()
        .and_then(|f| f.current_ratio.as_ref())
        .and_then(para_decimal_finito)
    {
        campos.insert("currentRatio".to_string(), val.round_dp(4));
    }

    let data_referencia_fonte = result
        .default_key_statistics
        .as_ref()
        .and_then(|k| k.most_recent_quarter.as_ref())
        .and_then(|d| d.fmt.clone())
        .filter(|s| !s.trim().is_empty());

    Ok(QuoteSummaryFundamentos {
        fonte: "YAHOO_FINANCE".to_string(),
        data_referencia_fonte,
        campos,
    })
}

fn para_decimal_finito(field: &YahooNumericField) -> Option<Decimal> {
    let raw = field.raw?;
    if !raw.is_finite() {
        return None;
    }
    Decimal::from_f64(raw)
}
