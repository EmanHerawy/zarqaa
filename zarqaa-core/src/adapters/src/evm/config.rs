// All the connection details for one EVM chain.
// We read from env vars so secrets never live in code.
pub struct EvmChainConfig {
    pub chain_id: String,
    pub rpc_url: String,
    pub explorer_api_url: String,
    pub explorer_api_key: String,
}

impl EvmChainConfig {
    pub fn ethereum_from_env() -> Self {
        Self {
            chain_id: "ethereum".to_string(),
            rpc_url: std::env::var("ZARQA_ETH_RPC_URL")
                .expect("ZARQA_ETH_RPC_URL must be set"),
            explorer_api_url: "https://api.etherscan.io/api".to_string(),
            explorer_api_key: std::env::var("ZARQA_ETHERSCAN_KEY")
                .unwrap_or_default(),
        }
    }
}
