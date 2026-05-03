use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZarqaError {
    #[error("TX hash not found: {0}")]
    TxNotFound(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("RPC timeout after {ms}ms")]
    RpcTimeout { ms: u64 },
    #[error("debug_traceTransaction not supported by this node")]
    TraceUnavailable,

    #[error("No adapter registered for chain: {0}")]
    ChainNotSupported(String),
    #[error("Stage not applicable on chain {chain}: {reason}")]
    StageNotApplicable { chain: String, reason: String },
    #[error("Invalid address format for chain {chain}: {address}")]
    InvalidAddress { chain: String, address: String },

    #[error("Source not verified on chain {chain} for {address}")]
    SourceNotVerified { chain: String, address: String },
    #[error("Explorer API error: {0}")]
    ExplorerApi(String),
    #[error("Explorer API rate limited")]
    ExplorerRateLimited,

    #[error("Could not resolve intent to calldata: {0}")]
    IntentUnresolvable(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ZarqaError {
    pub fn reason_code(&self) -> &'static str {
        match self {
            ZarqaError::TxNotFound(_)           => "UNRESOLVABLE_TX_HASH",
            ZarqaError::RpcTimeout { .. }       => "RPC_TIMEOUT",
            ZarqaError::TraceUnavailable        => "TRACE_UNAVAILABLE",
            ZarqaError::ChainNotSupported(_)    => "CHAIN_NOT_SUPPORTED",
            ZarqaError::StageNotApplicable {..} => "STAGE_NOT_APPLICABLE",
            ZarqaError::InvalidAddress {..}     => "ADDRESS_FORMAT_INVALID",
            ZarqaError::SourceNotVerified {..}  => "SOURCE_NOT_VERIFIED",
            ZarqaError::ExplorerRateLimited     => "EXPLORER_RATE_LIMITED",
            ZarqaError::IntentUnresolvable(_)   => "INTENT_UNRESOLVABLE",
            _                                   => "INTERNAL_ERROR",
        }
    }
}

pub type Result<T> = std::result::Result<T, ZarqaError>;
