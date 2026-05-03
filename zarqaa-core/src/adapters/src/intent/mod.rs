mod claude;

use reqwest::Client;
use serde::Deserialize;
use zarqaa_types::error::{Result, ZarqaaError};

// What Claude extracted from the raw user intent.
#[derive(Debug, Clone)]
pub struct NormalizedIntent {
    pub to: String,             // target contract address
    pub from: Option<String>,
    pub data: Option<String>,   // calldata hex
    pub value_wei: Option<u64>,
    pub chain: String,
    pub decoded_call: Option<String>, // Claude's interpretation of what's being called
}

// One contract address Claude identified in the call path.
#[derive(Debug, Clone)]
pub struct ExtractedAddress {
    pub address: Option<String>, // None for dynamic (resolved at runtime)
    pub addr_type: String,       // "static" | "dynamic"
    pub reason: String,
}

// Normalize any input form into a canonical transaction intent.
// Accepts raw tx objects, natural language, or anything in between.
pub async fn normalize(
    raw_input: &str,
    http: &Client,
    api_key: &str,
    api_url: &str,
    model: &str,
) -> Result<NormalizedIntent> {
    let prompt = format!(
        r#"You are a Web3 transaction parser. Extract transaction fields from the user input.
Return ONLY valid JSON, no explanation, no markdown.

Required format:
{{
  "to": "<hex address or null if unknown>",
  "from": "<hex address or null>",
  "data": "<0x calldata hex or null>",
  "value": "<decimal wei string or 0>",
  "chain": "<ethereum|arbitrum|base|polygon|optimism — default ethereum>",
  "decoded_call": "<your best guess at function + params, e.g. swap(address,uint256)>"
}}

User input:
{raw_input}"#
    );

    let text = claude::ask(http, api_key, api_url, model, prompt).await?;

    // Strip markdown fences if Claude included them
    let json_str = strip_json_fences(&text);

    #[derive(Deserialize)]
    struct Raw {
        to: Option<String>,
        from: Option<String>,
        data: Option<String>,
        value: Option<String>,
        chain: Option<String>,
        decoded_call: Option<String>,
    }

    let raw: Raw = serde_json::from_str(json_str)
        .map_err(|e| ZarqaaError::Internal(format!("Intent normalization parse failed: {e}\nClaude output: {text}")))?;

    let to = raw.to.filter(|s| !s.is_empty() && s != "null")
        .ok_or_else(|| ZarqaaError::Internal(
            "Could not extract a target contract address from the intent. \
             Please provide a contract address or a clearer description.".into()
        ))?;

    let value_wei = raw.value
        .as_deref()
        .and_then(|v| v.parse::<u64>().ok());

    Ok(NormalizedIntent {
        to,
        from: raw.from.filter(|s| !s.is_empty() && s != "null"),
        data: raw.data.filter(|s| !s.is_empty() && s != "null"),
        value_wei,
        chain: raw.chain.unwrap_or_else(|| "ethereum".into()),
        decoded_call: raw.decoded_call.filter(|s| !s.is_empty() && s != "null"),
    })
}

// Given the target contract's source code + the function being called,
// return every contract address that will be touched during execution.
pub async fn extract_addresses(
    source_code: &str,
    abi_json: &str,
    decoded_call: &str,
    http: &Client,
    api_key: &str,
    api_url: &str,
    model: &str,
) -> Vec<ExtractedAddress> {
    // Truncate very large source to fit comfortably in context
    let source_truncated = if source_code.len() > 40_000 {
        &source_code[..40_000]
    } else {
        source_code
    };

    let prompt = format!(
        r#"You are a Solidity security analyst performing static analysis.

Given a contract's source code and the specific function call being made,
identify every external contract address that will be CALLED (not just referenced)
during this transaction's execution.

Function call being made: {decoded_call}

Contract ABI (for context):
{abi_json}

Contract source code:
{source_truncated}

For each address, classify it:
- "static": known right now — hardcoded constant, or passed explicitly in calldata params
- "dynamic": resolved at runtime — read from a mapping, returned by a factory call, etc.

Return ONLY a JSON array, no explanation:
[
  {{"address": "0x...", "type": "static", "reason": "hardcoded WETH constant"}},
  {{"address": null, "type": "dynamic", "reason": "pool = factory.getPool(token0, token1, fee)"}}
]

Include ONLY addresses that are CALLED externally. Skip internal library calls and events."#
    );

    let text = match claude::ask(http, api_key, api_url, model, prompt).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("Static address extraction failed: {e}");
            return vec![];
        }
    };

    let json_str = strip_json_fences(&text);

    #[derive(Deserialize)]
    struct RawAddr {
        address: Option<String>,
        #[serde(rename = "type")]
        addr_type: String,
        reason: String,
    }

    match serde_json::from_str::<Vec<RawAddr>>(json_str) {
        Ok(items) => items
            .into_iter()
            .map(|r| ExtractedAddress {
                address: r.address.filter(|s| s.starts_with("0x") || s.starts_with("0X")),
                addr_type: r.addr_type,
                reason: r.reason,
            })
            .collect(),
        Err(e) => {
            tracing::warn!("Failed to parse extracted addresses: {e}\nClaude output: {text}");
            vec![]
        }
    }
}

fn strip_json_fences(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim()
}
