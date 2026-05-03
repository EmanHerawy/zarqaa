use reqwest::Client;
use serde::Deserialize;
use zarqaa_types::error::{Result, ZarqaaError};

// Etherscan's envelope: status "1" = success, "0" = error.
//
// IMPORTANT: when status is "0" (e.g. rate limit, invalid key), `result` is
// a plain String like "Max rate limit reached", NOT an array. If we deserialize
// `result` directly as Vec<SourceResult>, serde panics before we can check status.
// Solution: deserialize `result` as a raw JSON value, inspect status first,
// then parse the array only when we know it's safe.
#[derive(Deserialize)]
struct EtherscanEnvelope {
    status: String,
    message: String,
    result: serde_json::Value,
}

#[derive(Deserialize)]
struct SourceResult {
    #[serde(rename = "ABI")]
    abi: String,
    #[serde(rename = "ContractName")]
    contract_name: String,
    #[serde(rename = "SourceCode")]
    source_code: String,
}

pub struct ContractDetails {
    pub verified: bool,
    pub name: Option<String>,
    pub abi: Option<String>,    // JSON string, None if unverified
    pub source: Option<String>, // full source code, None if unverified
}

pub struct ExplorerClient {
    http: Client,
    api_url: String,
    api_key: String,
}

impl ExplorerClient {
    pub fn new(api_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            api_url: api_url.into(),
            api_key: api_key.into(),
        }
    }

    // Returns (verified, contract_name).
    //
    // How to detect "not verified": Etherscan returns status "1" even for
    // unverified contracts, but sets ABI to the literal string
    // "Contract source code not verified". That's the sentinel we check.
    pub async fn get_source_info(&self, address: &str) -> Result<(bool, Option<String>)> {
        // api_url already contains "?chainid=X", so we use "&" not "?" here
        let url = format!(
            "{}&module=contract&action=getsourcecode&address={}&apikey={}",
            self.api_url, address, self.api_key
        );

        let resp: EtherscanEnvelope = self.http.get(&url).send().await?.json().await?;

        // Check status BEFORE trying to parse result — result may be a String on error
        if resp.status != "1" {
            if resp.message.to_lowercase().contains("rate")
                || resp.result.as_str().map(|s| s.to_lowercase().contains("rate")).unwrap_or(false)
            {
                return Err(ZarqaaError::ExplorerRateLimited);
            }
            let detail = resp.result.as_str().unwrap_or(&resp.message).to_string();
            return Err(ZarqaaError::ExplorerApi(detail));
        }

        // Status is "1" — result is now safe to parse as an array
        let items: Vec<SourceResult> = serde_json::from_value(resp.result)?;
        let result = items.into_iter().next()
            .ok_or_else(|| ZarqaaError::ExplorerApi("empty result array".to_string()))?;

        let verified = result.abi != "Contract source code not verified";
        let name = if verified { Some(result.contract_name) } else { None };

        Ok((verified, name))
    }

    // Returns full source code + ABI for use in intent static analysis.
    // One Etherscan call gets everything (getsourcecode returns both).
    pub async fn get_contract_details(&self, address: &str) -> Result<ContractDetails> {
        let url = format!(
            "{}&module=contract&action=getsourcecode&address={}&apikey={}",
            self.api_url, address, self.api_key
        );

        let resp: EtherscanEnvelope = self.http.get(&url).send().await?.json().await?;

        if resp.status != "1" {
            if resp.message.to_lowercase().contains("rate")
                || resp.result.as_str().map(|s| s.to_lowercase().contains("rate")).unwrap_or(false)
            {
                return Err(ZarqaaError::ExplorerRateLimited);
            }
            let detail = resp.result.as_str().unwrap_or(&resp.message).to_string();
            return Err(ZarqaaError::ExplorerApi(detail));
        }

        let items: Vec<SourceResult> = serde_json::from_value(resp.result)?;
        let result = items.into_iter().next()
            .ok_or_else(|| ZarqaaError::ExplorerApi("empty result array".to_string()))?;

        let verified = result.abi != "Contract source code not verified";
        if !verified {
            return Ok(ContractDetails { verified: false, name: None, abi: None, source: None });
        }

        // Source code may be wrapped in Etherscan's multi-file JSON envelope —
        // strip the outer {{ }} if present and return the raw content.
        let source = if result.source_code.starts_with("{{") {
            result.source_code.trim_start_matches('{').trim_end_matches('}').to_string()
        } else {
            result.source_code
        };

        Ok(ContractDetails {
            verified: true,
            name: Some(result.contract_name),
            abi: Some(result.abi),
            source: Some(source),
        })
    }
}
