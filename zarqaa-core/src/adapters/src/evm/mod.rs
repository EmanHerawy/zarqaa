pub mod config;
pub mod explorer;
pub mod infra;
pub mod rpc;

use config::EvmChainConfig;
use explorer::ExplorerClient;
use rpc::EvmRpcClient;
use zarqaa_types::error::Result;
use zarqaa_types::report::{ChainAddress, LegReport, Verdict};

pub struct EvmAdapter {
    rpc: EvmRpcClient,
    explorer: ExplorerClient,
    chain_id: String,
}

impl EvmAdapter {
    pub fn new(config: EvmChainConfig) -> Self {
        Self {
            rpc: EvmRpcClient::new(&config.rpc_url, &config.chain_id),
            explorer: ExplorerClient::new(&config.explorer_api_url, &config.explorer_api_key),
            chain_id: config.chain_id,
        }
    }

    // Stage 1: resolve a tx hash into a list of contract addresses.
    pub async fn resolve_legs(&self, tx_hash: &str) -> Result<Vec<String>> {
        let legs: Vec<ChainAddress> = self.rpc.resolve_tx_path(tx_hash).await?;
        Ok(legs.into_iter().map(|l| l.address).collect())
    }

    // Stages 2-4: analyze one contract address and produce a LegReport.
    //
    // Important design choice: this method never returns Err.
    // If a check fails (e.g. Etherscan is down), we record the failure
    // as a note and keep going. "Silence = danger" — a failed check
    // becomes Unverified, not a crash.
    pub async fn analyze_leg(&self, address: &str) -> LegReport {
        let mut notes = Vec::new();
        let mut verdict = Verdict::Green;

        // --- Stage 2a: source verification ---
        let (source_verified, contract_name) =
            match self.explorer.get_source_info(address).await {
                Ok(v) => v,
                Err(e) => {
                    notes.push(format!("Source check failed: {e}"));
                    // Can't verify = treat as unverified
                    (false, None)
                }
            };

        if !source_verified {
            notes.push("Source code not verified on Etherscan".to_string());
            verdict = Verdict::Unverified;
        } else if let Some(name) = &contract_name {
            notes.push(format!("Verified contract: {name}"));
        }

        // --- Stage 2b: proxy detection ---
        let (is_proxy, proxy_implementation) =
            match self.rpc.get_proxy_implementation(address).await {
                Ok(Some(impl_addr)) => {
                    notes.push(format!("ERC-1967 proxy → impl: {impl_addr}"));
                    (true, Some(impl_addr))
                }
                Ok(None) => (false, None),
                Err(e) => {
                    notes.push(format!("Proxy check failed: {e}"));
                    (false, None)
                }
            };

        // --- Stage 2c: known infra detection + bridge security card ---
        let infra_kind = infra::known_infra_label(&self.chain_id, address);
        let bridge_info = infra::bridge_mock_info(&self.chain_id, address);

        if let Some(label) = &infra_kind {
            notes.push(format!("Known infrastructure: {label}"));
            if verdict == Verdict::Green {
                verdict = Verdict::Amber;
            }
        }

        if let Some(info) = &bridge_info {
            // Escalate verdict based on bridge card data
            if info.past_exploits_usd.unwrap_or(0) > 0 && verdict != Verdict::Red {
                verdict = Verdict::Amber;
            }
            if info.destination_chain.is_some() {
                notes.push(format!(
                    "Cross-chain destination tracking not yet supported for {} \
                     (CROSS_CHAIN_UNSUPPORTED)",
                    info.protocol
                ));
            }
        }

        LegReport {
            address: address.to_string(),
            chain: self.chain_id.clone(),
            verdict,
            source_verified,
            is_proxy,
            proxy_implementation,
            infra_kind,
            bridge_info,
            notes,
        }
    }
}
