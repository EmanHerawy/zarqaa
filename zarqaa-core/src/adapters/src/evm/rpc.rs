use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use zarqa_types::error::{Result, ZarqaError};
use zarqa_types::report::ChainAddress;

// Represents one node in the call tree returned by debug_traceTransaction.
// Each frame is one contract call; `calls` are the inner calls it made.
#[derive(serde::Deserialize)]
struct CallFrame {
    to: Option<String>,
    calls: Option<Vec<CallFrame>>,
}

// Recursively walks the call tree and collects every unique contract address.
// Uses a HashSet to deduplicate — same contract can appear in many frames.
fn collect_addresses(
    frame: &CallFrame,
    seen: &mut std::collections::HashSet<String>,
    out: &mut Vec<String>,
) {
    if let Some(to) = &frame.to {
        let addr = to.to_lowercase();
        if seen.insert(addr.clone()) {
            out.push(addr);
        }
    }
    if let Some(calls) = &frame.calls {
        for call in calls {
            collect_addresses(call, seen, out);
        }
    }
}

pub struct EvmRpcClient {
    pub rpc_url: String,
    pub chain_id: String,
}

impl EvmRpcClient {
    pub fn new(rpc_url: impl Into<String>, chain_id: impl Into<String>) -> Self {
        Self { rpc_url: rpc_url.into(), chain_id: chain_id.into() }
    }

    fn provider(&self) -> Result<impl Provider> {
        let url = self.rpc_url.parse()
            .map_err(|e| ZarqaError::Internal(format!("{e}")))?;
        Ok(ProviderBuilder::new().connect_http(url))
    }

    // Given a tx hash, return every contract address the transaction touched.
    //
    // Uses debug_traceTransaction (callTracer) — the complete EVM call graph.
    // This captures internal delegatecalls and non-event-emitting contracts
    // that the receipt-log approach would silently miss.
    //
    // TODO: if debug_traceTransaction fails (TRACE_UNAVAILABLE), fall back to
    // tx.to + receipt log emitters. Incomplete but better than hard-failing.
    pub async fn resolve_tx_path(&self, tx_hash: &str) -> Result<Vec<ChainAddress>> {
        let hash: B256 = tx_hash.parse()
            .map_err(|_| ZarqaError::TxNotFound(tx_hash.to_string()))?;

        let provider = self.provider()?;

        let trace: serde_json::Value = provider
            .raw_request(
                "debug_traceTransaction".into(),
                (hash, serde_json::json!({ "tracer": "callTracer" })),
            )
            .await
            .map_err(|e| ZarqaError::Rpc(e.to_string()))?;

        let root: CallFrame = serde_json::from_value(trace)
            .map_err(|e| ZarqaError::Internal(format!("trace parse failed: {e}")))?;

        let mut seen = std::collections::HashSet::new();
        let mut addresses = Vec::new();
        collect_addresses(&root, &mut seen, &mut addresses);

        Ok(addresses
            .into_iter()
            .map(|addr| ChainAddress { chain: self.chain_id.clone(), address: addr })
            .collect())
    }

    // Read ERC-1967 proxy implementation slot.
    // Returns None if the slot is empty (contract is not a proxy).
    //
    // EIP-1967 reserves slot keccak256("eip1967.proxy.implementation") - 1
    // as a standard location for proxy implementation addresses.
    pub async fn get_proxy_implementation(&self, address: &str) -> Result<Option<String>> {
        const SLOT: &str = "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";

        let addr: Address = address.parse()
            .map_err(|_| ZarqaError::Internal(format!("invalid address: {address}")))?;
        let slot: B256 = SLOT.parse()
            .map_err(|_| ZarqaError::Internal("bad slot constant".to_string()))?;

        let provider = self.provider()?;
        let raw = provider.get_storage_at(addr, slot.into()).await
            .map_err(|e| ZarqaError::Rpc(e.to_string()))?;

        if raw == U256::ZERO {
            return Ok(None);
        }

        // Address is 20 bytes, stored right-aligned in a 32-byte slot.
        let bytes: [u8; 32] = raw.to_be_bytes();
        let impl_addr = Address::from_slice(&bytes[12..]);
        Ok(Some(format!("{impl_addr:#x}")))
    }
}
