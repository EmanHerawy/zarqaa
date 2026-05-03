use reqwest::Client;
use serde::Deserialize;
use zarqaa_types::error::{Result, ZarqaaError};

// Send a single-turn prompt to any OpenAI-compatible API and return the text response.
// Supports OpenRouter, OpenAI, Groq, Together, Ollama, etc.
// Times out after 30 seconds.
pub async fn ask(
    http: &Client,
    api_key: &str,
    api_url: &str,
    model: &str,
    prompt: String,
) -> Result<String> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 2048,
        "messages": [{"role": "user", "content": prompt}]
    });

    let resp = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        http.post(api_url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send(),
    )
    .await
    .map_err(|_| ZarqaaError::Internal("LLM API timed out".into()))?
    .map_err(|e| ZarqaaError::Internal(format!("LLM API request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ZarqaaError::Internal(format!("LLM API {status}: {body}")));
    }

    #[derive(Deserialize)]
    struct MsgContent {
        content: String,
    }
    #[derive(Deserialize)]
    struct Choice {
        message: MsgContent,
    }
    #[derive(Deserialize)]
    struct Response {
        choices: Vec<Choice>,
    }

    let parsed: Response = resp
        .json()
        .await
        .map_err(|e| ZarqaaError::Internal(format!("LLM response parse failed: {e}")))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| ZarqaaError::Internal("LLM returned empty choices".into()))
}
