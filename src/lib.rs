//! Biblioteca compartilhada `mbr-yfinance` para consumo de cotações (`chart`) e
//! fundamentos (`quoteSummary` com cookie+crumb) do Yahoo Finance em `Decimal`.
#![forbid(unsafe_code)]
#![deny(clippy::correctness, clippy::unwrap_used)]

#[cfg(feature = "client")]
pub mod client;
pub mod error;
pub mod parser;
pub mod types;

#[cfg(feature = "client")]
pub use client::YahooFinanceClient;
pub use error::YfinanceError;
pub use parser::{normalizar_ticker_b3, parse_chart_json, parse_quote_summary_json};
pub use types::{PregaoChart, QuoteSummaryFundamentos};
