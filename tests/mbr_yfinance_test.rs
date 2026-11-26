use mbr_yfinance::{normalizar_ticker_b3, parse_chart_json, parse_quote_summary_json};
use rust_decimal::Decimal;

#[test]
fn normaliza_ticker_b3_para_sufixo_sa() {
    assert_eq!(normalizar_ticker_b3("cxse3"), "CXSE3.SA");
    assert_eq!(normalizar_ticker_b3("PETR4.SA"), "PETR4.SA");
}

#[test]
fn parse_chart_json_extrai_pregoes_em_decimal() {
    let json = r#"{
        "chart": {
            "result": [{
                "timestamp": [1700000000, 1700086400],
                "indicators": {
                    "quote": [{
                        "close": [20.15, 20.30],
                        "volume": [1000.0, 1500.0]
                    }]
                }
            }]
        }
    }"#;
    let pregoes = parse_chart_json("CXSE3", json).expect("chart valido");
    assert_eq!(pregoes.len(), 2);
    assert_eq!(pregoes[1].close.round_dp(2), Decimal::new(2030, 2));
}

#[test]
fn parse_quote_summary_json_extrai_indicadores_em_decimal() {
    let json = r#"{
        "quoteSummary": {
            "result": [{
                "summaryDetail": { "trailingPE": { "raw": 13.355263 } },
                "defaultKeyStatistics": {
                    "priceToBook": { "raw": 4.3468947 },
                    "mostRecentQuarter": { "fmt": "2026-09-30" }
                },
                "financialData": {
                    "returnOnEquity": { "raw": 0.33157003 },
                    "profitMargins": { "raw": 0.75847 },
                    "currentRatio": { "raw": 1.414 }
                }
            }]
        }
    }"#;
    let parsed = parse_quote_summary_json("CXSE3", json).expect("summary valido");
    assert_eq!(parsed.fonte, "YAHOO_FINANCE");
    assert_eq!(parsed.data_referencia_fonte.as_deref(), Some("2026-09-30"));
    assert_eq!(
        parsed.campos.get("trailingPE"),
        Some(&Decimal::new(1336, 2))
    );
    assert_eq!(
        parsed.campos.get("priceToBook"),
        Some(&Decimal::new(435, 2))
    );
    assert_eq!(
        parsed.campos.get("returnOnEquity"),
        Some(&Decimal::new(3316, 4))
    );
    assert_eq!(
        parsed.campos.get("profitMargins"),
        Some(&Decimal::new(7585, 4))
    );
    assert_eq!(
        parsed.campos.get("currentRatio"),
        Some(&Decimal::new(14140, 4))
    );
}
