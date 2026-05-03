use std::collections::HashMap;
use std::sync::Arc;
use zarqaa_adapters::{EvmAdapter, EvmChainConfig};

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub adapters: HashMap<String, EvmAdapter>,
    pub anthropic_key: Option<String>,
}

impl AppState {
    pub fn from_env() -> SharedState {
        let mut adapters = HashMap::new();

        if std::env::var("ZARQA_ETH_RPC_URL").is_ok() {
            adapters.insert(
                "ethereum".to_string(),
                EvmAdapter::new(EvmChainConfig::ethereum_from_env()),
            );
            tracing::info!("Chain configured: ethereum");
        } else {
            tracing::warn!("ZARQA_ETH_RPC_URL not set — ethereum chain unavailable");
        }

        if adapters.is_empty() {
            tracing::error!("No chains configured — set ZARQA_ETH_RPC_URL");
        }

        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
        if anthropic_key.is_none() {
            tracing::warn!("ANTHROPIC_API_KEY not set — /analyze-intent endpoint will return 503");
        }

        Arc::new(Self { adapters, anthropic_key })
    }
}
