use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::report::ChainAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub verified: bool,
    pub source_code: Option<String>,
    pub abi: Option<String>,
    pub compiler_version: Option<String>,
    pub contract_name: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub is_proxy: bool,
    pub implementation: Option<String>,
    pub proxy_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerType {
    Eoa,
    Multisig,
    Dao,
    Contract,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    pub owner_address: Option<String>,
    pub owner_type: OwnerType,
    pub has_timelock: bool,
    pub timelock_delay_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub auditor: String,
    pub date: String,
    pub report_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    pub audits: Vec<AuditEntry>,
    pub last_audit_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitEvent {
    pub date: String,
    pub description: String,
    pub loss_usd: Option<u64>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitHistory {
    pub events: Vec<ExploitEvent>,
    pub lookback_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevInput {
    pub tx_hash: Option<String>,
    pub target_address: String,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevReport {
    pub sandwich_risk: bool,
    pub flashbots_detected: bool,
    pub estimated_mev_loss_usd: Option<f64>,
    pub details: Vec<String>,
}

#[async_trait]
pub trait ChainAdapter: Send + Sync {
    fn chain_id(&self) -> &str;

    fn validate_address(&self, address: &str) -> bool;
    fn validate_tx_hash(&self, hash: &str) -> bool;
    fn normalize_address(&self, address: &str) -> String;

    async fn resolve_tx_path(&self, tx_hash: &str) -> Result<Vec<ChainAddress>>;
    async fn get_source_info(&self, address: &str) -> Result<SourceInfo>;
    async fn detect_proxy(&self, address: &str) -> Result<ProxyInfo>;
    async fn get_ownership(&self, address: &str) -> Result<OwnershipInfo>;
    async fn get_audits(&self, address: &str) -> Result<AuditInfo>;
    async fn get_exploit_history(&self, address: &str, lookback_days: u32) -> Result<ExploitHistory>;
    async fn analyze_mev(&self, input: &MevInput) -> Result<MevReport>;
    async fn detect_infra(&self, address: &str) -> Result<Option<crate::infra::InfraReport>>;
}

#[derive(Default)]
pub struct ChainRegistry {
    adapters: HashMap<String, Arc<dyn ChainAdapter>>,
}

impl ChainRegistry {
    pub fn register(&mut self, adapter: Arc<dyn ChainAdapter>) {
        self.adapters.insert(adapter.chain_id().to_string(), adapter);
    }

    pub fn get(&self, chain_id: &str) -> Option<Arc<dyn ChainAdapter>> {
        self.adapters.get(chain_id).cloned()
    }

    pub fn is_registered(&self, chain_id: &str) -> bool {
        self.adapters.contains_key(chain_id)
    }

    pub fn registered_chains(&self) -> Vec<&str> {
        self.adapters.keys().map(String::as_str).collect()
    }
}
