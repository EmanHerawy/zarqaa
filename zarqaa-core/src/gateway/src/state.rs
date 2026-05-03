use std::collections::HashMap;
use std::sync::Arc;
use zarqaa_adapters::{EvmAdapter, EvmChainConfig};

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub adapters: HashMap<String, EvmAdapter>,
}

impl AppState {
    pub fn from_env() -> SharedState {
        let mut adapters = HashMap::new();

        // Ethereum mainnet — always loaded if env vars are present
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

        Arc::new(Self { adapters })
    }
}
