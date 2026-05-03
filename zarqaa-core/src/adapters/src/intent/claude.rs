use reqwest::Client;
use serde::{Deserialize, Serialize};
use zarqaa_types::error::{Result, ZarqaaError};

const CLAUDE_API: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-sonnet-4-6";

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct Request {
    model: &'static str,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Response {
    content: Vec<ContentBlock>,
}

// Send a single-turn prompt to Claude and return the text response.
// Times out after 30 seconds. Returns ZarqaaError::Internal on failure.
pub async fn ask(http: &Client, api_key: &str, prompt: String) -> Result<String> {
    let body = Request {
        model: MODEL,
        max_tokens: 2048,
        messages: vec![Message { role: "user", content: prompt }],
    };

    let resp = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        http.post(CLAUDE_API)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send(),
    )
    .await
    .map_err(|_| ZarqaaError::Internal("Claude API timed out".into()))?
    .map_err(|e| ZarqaaError::Internal(format!("Claude API request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ZarqaaError::Internal(format!("Claude API {status}: {body}")));
    }

    let parsed: Response = resp
        .json()
        .await
        .map_err(|e| ZarqaaError::Internal(format!("Claude response parse failed: {e}")))?;

    parsed
        .content
        .into_iter()
        .next()
        .map(|c| c.text)
        .ok_or_else(|| ZarqaaError::Internal("Claude returned empty content".into()))
}
