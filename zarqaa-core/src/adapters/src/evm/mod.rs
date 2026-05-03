pub mod config;
pub mod explorer;
pub mod feeds;
pub mod infra;
pub mod rpc;

use config::EvmChainConfig;
use explorer::ExplorerClient;
use reqwest::Client;
use rpc::EvmRpcClient;
use zarqaa_types::error::Result;
use zarqaa_types::report::{ChainAddress, DataSource, LegReport, Verdict};

pub struct EvmAdapter {
    rpc: EvmRpcClient,
    explorer: ExplorerClient,
    http: Client,
    chain_id: String,
}

impl EvmAdapter {
    pub fn new(config: EvmChainConfig) -> Self {
        Self {
            rpc: EvmRpcClient::new(&config.rpc_url, &config.chain_id),
            explorer: ExplorerClient::new(&config.explorer_api_url, &config.explorer_api_key),
            http: Client::new(),
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
    // Never returns Err. If a check fails, we record the failure as a note and keep
    // going. "Silence = danger" — a failed check becomes Unverified, not a crash.
    pub async fn analyze_leg(&self, address: &str) -> LegReport {
        // --- EVM precompile fast-path ---
        // Addresses 0x01-0xff are system precompiles. No source code exists by design —
        // treating them as "unverified" would poison the route verdict incorrectly.
        if let Some(precompile_name) = evm_precompile_name(address) {
            return LegReport {
                address: address.to_string(),
                chain: self.chain_id.clone(),
                verdict: Verdict::Green,
                source_verified: true,
                is_proxy: false,
                proxy_implementation: None,
                infra_kind: Some(format!("EVM precompile ({precompile_name})")),
                bridge_info: None,
                notes: vec![
                    format!("System precompile: {precompile_name}"),
                    "Precompiles are built into the EVM — no source code exists and none is needed".to_string(),
                ],
            };
        }

        let mut notes = Vec::new();
        let mut verdict = Verdict::Green;

        // --- Stage 2a: source verification ---
        let (source_verified, contract_name) =
            match self.explorer.get_source_info(address).await {
                Ok(v) => v,
                Err(e) => {
                    notes.push(format!("Source check failed: {e}"));
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
        let mut bridge_info = infra::bridge_static_info(&self.chain_id, address);

        if let Some(label) = &infra_kind {
            notes.push(format!("Known infrastructure: {label}"));
            if verdict == Verdict::Green {
                verdict = Verdict::Amber;
            }
        }

        // --- Stage 2d: fetch live recent flags for bridge protocols ---
        if let Some(ref mut info) = bridge_info {
            let (live_flags, flags_source) =
                feeds::fetch_recent_flags(&info.protocol, &self.http).await;

            // Live fetch succeeded — replace static placeholder with real data
            if flags_source != DataSource::Static {
                info.recent_flags = live_flags;
                info.recent_flags_source = flags_source;
            }

            // Escalate verdict based on bridge card data
            if info.past_exploits_usd.unwrap_or(0) > 0 && verdict != Verdict::Red {
                verdict = Verdict::Amber;
            }
            if !info.recent_flags.is_empty() && verdict == Verdict::Green {
                verdict = Verdict::Amber;
            }
            if let Some(dest) = &info.destination_chain {
                notes.push(format!(
                    "Cross-chain via {} → {dest} (destination verification coming soon)",
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

// Returns the human-readable name of an EVM precompile, or None if address is not one.
// Standard precompiles: 0x01-0x09. Some chains extend this; we cover up to 0xff.
fn evm_precompile_name(address: &str) -> Option<&'static str> {
    let stripped = address.trim_start_matches("0x").trim_start_matches("0X");
    // All-zero prefix except the last two hex chars
    let all_but_last = stripped.get(..stripped.len().saturating_sub(2)).unwrap_or("");
    if !all_but_last.chars().all(|c| c == '0') {
        return None;
    }
    let last_byte = u8::from_str_radix(stripped.get(stripped.len().saturating_sub(2)..).unwrap_or("00"), 16).ok()?;
    if last_byte == 0 {
        return None;
    }
    Some(match last_byte {
        0x01 => "ecRecover",
        0x02 => "SHA-256",
        0x03 => "RIPEMD-160",
        0x04 => "identity (data copy)",
        0x05 => "modexp",
        0x06 => "ecAdd (alt_bn128)",
        0x07 => "ecMul (alt_bn128)",
        0x08 => "ecPairing (alt_bn128)",
        0x09 => "blake2f",
        _    => "system precompile",
    })
}
