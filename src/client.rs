use std::time::Duration;

use reqwest::Client;
use tokio::sync::RwLock;

use crate::{
    PregaoChart, QuoteSummaryFundamentos, YfinanceError, normalizar_ticker_b3, parse_chart_json,
    parse_quote_summary_json,
};

/// Cliente HTTP assíncrono compartilhado para Yahoo Finance (`chart` e `quoteSummary` com cookie+crumb).
pub struct YahooFinanceClient {
    client: Client,
    base_url: String,
    cookie_url: String,
    crumb: RwLock<Option<String>>,
}

impl YahooFinanceClient {
    /// Cria um cliente configurado com `cookie_store(true)` e TLS nativo `rustls`.
    ///
    /// # Errors
    /// Retorna [`YfinanceError::ProvedorIndisponivel`] se o cliente HTTP não puder ser construído.
    pub fn new(base_url: &str, cookie_url: &str) -> Result<Self, YfinanceError> {
        let client = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            cookie_url: cookie_url.trim_end_matches('/').to_string(),
            crumb: RwLock::new(None),
        })
    }

    /// Busca série histórica diária (`range=3mo&interval=1d`) em `v8/finance/chart`.
    ///
    /// # Errors
    /// Retorna [`YfinanceError::AtivoNaoEncontrado`] (HTTP 404 ou sem pregões) ou [`YfinanceError::ProvedorIndisponivel`].
    pub async fn buscar_chart_3mo(&self, ticker: &str) -> Result<Vec<PregaoChart>, YfinanceError> {
        let symbol = normalizar_ticker_b3(ticker);
        let url = format!(
            "{}/v8/finance/chart/{symbol}?range=3mo&interval=1d",
            self.base_url
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))?;

        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(YfinanceError::AtivoNaoEncontrado(ticker.to_string()));
        }
        if !status.is_success() {
            return Err(YfinanceError::ProvedorIndisponivel(format!(
                "HTTP {status} ao consultar {symbol}"
            )));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))?;
        parse_chart_json(ticker, &body)
    }

    /// Busca fundamentos em `v10/finance/quoteSummary` com renovação automática de cookie+crumb.
    ///
    /// # Errors
    /// Retorna [`YfinanceError::ProvedorIndisponivel`] ou [`YfinanceError::AtivoNaoEncontrado`].
    pub async fn buscar_quote_summary(
        &self,
        ticker: &str,
    ) -> Result<QuoteSummaryFundamentos, YfinanceError> {
        let symbol = normalizar_ticker_b3(ticker);
        let crumb = self.obter_crumb(false).await?;
        let mut resp = self.enviar_quote_summary(&symbol, &crumb).await?;

        if resp.status().as_u16() == 401 {
            let novo_crumb = self.obter_crumb(true).await?;
            resp = self.enviar_quote_summary(&symbol, &novo_crumb).await?;
        }

        if resp.status().as_u16() == 404 {
            return Err(YfinanceError::AtivoNaoEncontrado(ticker.to_string()));
        }
        if !resp.status().is_success() {
            return Err(YfinanceError::ProvedorIndisponivel(format!(
                "Yahoo quoteSummary HTTP {} para {symbol}",
                resp.status()
            )));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))?;
        parse_quote_summary_json(ticker, &body)
    }

    async fn obter_crumb(&self, forcar_renovacao: bool) -> Result<String, YfinanceError> {
        if !forcar_renovacao && let Some(c) = self.crumb.read().await.clone() {
            return Ok(c);
        }

        let _ = self.client.get(&self.cookie_url).send().await;
        let crumb_url = format!("{}/v1/test/getcrumb", self.base_url);
        let resp = self
            .client
            .get(&crumb_url)
            .send()
            .await
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(YfinanceError::ProvedorIndisponivel(format!(
                "Yahoo crumb HTTP {}",
                resp.status()
            )));
        }

        let crumb = resp
            .text()
            .await
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))?
            .trim()
            .to_string();
        if crumb.is_empty() {
            return Err(YfinanceError::ProvedorIndisponivel(
                "Yahoo crumb vazio".to_string(),
            ));
        }

        *self.crumb.write().await = Some(crumb.clone());
        Ok(crumb)
    }

    async fn enviar_quote_summary(
        &self,
        symbol: &str,
        crumb: &str,
    ) -> Result<reqwest::Response, YfinanceError> {
        let url = format!(
            "{}/v10/finance/quoteSummary/{symbol}?modules=defaultKeyStatistics,financialData,summaryDetail&crumb={crumb}",
            self.base_url
        );
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| YfinanceError::ProvedorIndisponivel(e.to_string()))
    }
}
