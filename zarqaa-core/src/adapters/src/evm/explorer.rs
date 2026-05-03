use reqwest::Client;
use serde::Deserialize;
use zarqa_types::error::{Result, ZarqaError};

// Etherscan wraps every response in this envelope.
// `status` is "1" for success, "0" for error.
#[derive(Deserialize)]
struct EtherscanResp {
    status: String,
    message: String,
    result: Vec<SourceResult>,
}

#[derive(Deserialize)]
struct SourceResult {
    #[serde(rename = "ABI")]
    abi: String,
    #[serde(rename = "ContractName")]
    contract_name: String,
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
        let url = format!(
            "{}?module=contract&action=getsourcecode&address={}&apikey={}",
            self.api_url, address, self.api_key
        );

        let resp: EtherscanResp = self.http.get(&url).send().await?.json().await?;

        if resp.status != "1" {
            // Etherscan uses "Max rate limit reached" in the message for rate limits
            if resp.message.to_lowercase().contains("rate") {
                return Err(ZarqaError::ExplorerRateLimited);
            }
            return Err(ZarqaError::ExplorerApi(resp.message));
        }

        let result = resp.result.into_iter().next()
            .ok_or_else(|| ZarqaError::ExplorerApi("empty result".to_string()))?;

        let verified = result.abi != "Contract source code not verified";
        let name = if verified { Some(result.contract_name) } else { None };

        Ok((verified, name))
    }
}
