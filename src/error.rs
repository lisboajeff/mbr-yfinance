use thiserror::Error;

/// Erros retornados pela biblioteca `mbr-yfinance`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum YfinanceError {
    #[error("ativo não encontrado no Yahoo Finance: {0}")]
    AtivoNaoEncontrado(String),
    #[error("provedor Yahoo Finance indisponível: {0}")]
    ProvedorIndisponivel(String),
    #[error("resposta inválida do Yahoo Finance: {0}")]
    RespostaInvalida(String),
}
