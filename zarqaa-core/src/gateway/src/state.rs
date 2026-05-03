use std::collections::HashMap;
use std::sync::Arc;
use zarqaa_adapters::{EvmAdapter, EvmChainConfig, LlmConfig};

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub adapters: HashMap<String, EvmAdapter>,
    pub llm: Option<LlmConfig>,
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

        // LLM config: LLM_API_KEY (falls back to ANTHROPIC_API_KEY for compat)
        // LLM_API_URL: e.g. https://openrouter.ai/api/v1/chat/completions
        // LLM_MODEL:   e.g. anthropic/claude-sonnet-4-5 or openai/gpt-4o
        let llm_key = std::env::var("LLM_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .ok();

        let llm = llm_key.map(|api_key| {
            let api_url = std::env::var("LLM_API_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".into());
            let model = std::env::var("LLM_MODEL")
                .unwrap_or_else(|_| "anthropic/claude-sonnet-4-5".into());
            tracing::info!(api_url = %api_url, model = %model, "LLM configured");
            LlmConfig { api_key, api_url, model }
        });

        if llm.is_none() {
            tracing::warn!("LLM_API_KEY / ANTHROPIC_API_KEY not set — /analyze-intent will return 503");
        }

        Arc::new(Self { adapters, llm })
    }
}
