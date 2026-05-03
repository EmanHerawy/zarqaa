pub mod evm;
pub mod intent;
pub mod mev;

pub use evm::{config::EvmChainConfig, EvmAdapter};

pub struct LlmConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
}
