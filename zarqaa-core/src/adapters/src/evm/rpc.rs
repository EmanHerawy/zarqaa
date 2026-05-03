use alloy::consensus::Transaction;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use zarqa_types::error::{Result, ZarqaError};
use zarqa_types::report::ChainAddress;

pub struct EvmRpcClient {
    pub rpc_url: String,
    pub chain_id: String,
}

impl EvmRpcClient {
    pub fn new(rpc_url: impl Into<String>, chain_id: impl Into<String>) -> Self {
        Self { rpc_url: rpc_url.into(), chain_id: chain_id.into() }
    }

    // Build an alloy provider from our RPC URL.
    // alloy's ProviderBuilder handles HTTP transport, retries, etc.
    fn provider(&self) -> Result<impl Provider> {
        let url = self.rpc_url.parse()
            .map_err(|e| ZarqaError::Internal(format!("{e}")))?;
        Ok(ProviderBuilder::new().connect_http(url))
    }

    // Given a tx hash, return every contract address the transaction touched.
    //
    // How it works:
    //   1. Fetch the transaction itself — the `to` field is the first contract.
    //   2. Fetch the receipt — each log entry was emitted by a contract.
    //      The log's `address` field is that contract's address.
    //   3. Deduplicate (same contract can emit multiple logs).
    pub async fn resolve_tx_path(&self, tx_hash: &str) -> Result<Vec<ChainAddress>> {
        let hash: B256 = tx_hash.parse()
            .map_err(|_| ZarqaError::TxNotFound(tx_hash.to_string()))?;

        let provider = self.provider()?;

        let tx = provider.get_transaction_by_hash(hash).await
            .map_err(|e| ZarqaError::Rpc(e.to_string()))?
            .ok_or_else(|| ZarqaError::TxNotFound(tx_hash.to_string()))?;

        let receipt = provider.get_transaction_receipt(hash).await
            .map_err(|e| ZarqaError::Rpc(e.to_string()))?
            .ok_or_else(|| ZarqaError::TxNotFound(tx_hash.to_string()))?;

        let mut seen = std::collections::HashSet::new();
        let mut legs: Vec<ChainAddress> = Vec::new();

        // Helper closure — adds an address only if we haven't seen it yet
        let chain_id = &self.chain_id;
        let mut push = |addr: String| {
            if seen.insert(addr.clone()) {
                legs.push(ChainAddress { chain: chain_id.clone(), address: addr });
            }
        };

        // First leg: the direct target of the transaction.
        // In alloy 1.x, tx fields live in tx.inner and `to()` returns TxKind,
        // which is either Create (contract deployment) or Call(address).
        if let Some(to) = tx.inner.to() {
            push(format!("{to:#x}"));
        }

        // Inner legs: every contract that fired an event during execution.
        // In alloy 1.x, logs() is a method on the receipt envelope, not a field.
        for log in receipt.inner.logs() {
            push(format!("{:#x}", log.address()));
        }

        Ok(legs)
    }

    // Read ERC-1967 proxy implementation slot.
    // Returns None if the slot is empty (contract is not a proxy).
    //
    // Why this slot? EIP-1967 defines a standard storage location for proxy
    // contracts to store their implementation address. The slot value is:
    //   bytes32(uint256(keccak256("eip1967.proxy.implementation")) - 1)
    // Using a pseudo-random slot avoids collisions with the contract's own storage.
    pub async fn get_proxy_implementation(&self, address: &str) -> Result<Option<String>> {
        const SLOT: &str = "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";

        let addr: Address = address.parse()
            .map_err(|_| ZarqaError::Internal(format!("invalid address: {address}")))?;
        let slot: B256 = SLOT.parse()
            .map_err(|_| ZarqaError::Internal("bad slot constant".to_string()))?;

        let provider = self.provider()?;
        let raw = provider.get_storage_at(addr, slot.into()).await
            .map_err(|e| ZarqaError::Rpc(e.to_string()))?;

        // A zero value means nothing is stored there — not a proxy
        if raw == U256::ZERO {
            return Ok(None);
        }

        // The implementation address is stored right-aligned in the 32-byte slot.
        // An address is 20 bytes, so it sits in bytes [12..32].
        let bytes: [u8; 32] = raw.to_be_bytes();
        let impl_addr = Address::from_slice(&bytes[12..]);
        Ok(Some(format!("{impl_addr:#x}")))
    }
}
