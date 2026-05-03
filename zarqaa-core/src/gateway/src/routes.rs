use axum::{extract::State, http::StatusCode, Json};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use zarqaa_adapters::mev;
use zarqaa_types::report::RouteReport;

use crate::state::SharedState;

// ── Shared response envelope ───────────────────────────────────────────────

#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse {
    Ok(RouteReport),
    Err(ApiError),
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

fn err(status: StatusCode, error: impl Into<String>, code: &str) -> (StatusCode, Json<ApiResponse>) {
    (status, Json(ApiResponse::Err(ApiError { error: error.into(), code: code.into() })))
}

// ── POST /analyze — tx hash path ──────────────────────────────────────────

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub tx_hash: String,
    pub chain: String,
}

pub async fn analyze(
    State(state): State<SharedState>,
    Json(req): Json<AnalyzeRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let adapter = match state.adapters.get(&req.chain) {
        Some(a) => a,
        None => return err(StatusCode::BAD_REQUEST,
            format!("Chain '{}' not configured", req.chain), "UNSUPPORTED_CHAIN"),
    };

    tracing::info!(tx_hash = %req.tx_hash, chain = %req.chain, "resolving legs");

    let addresses = match adapter.resolve_legs(&req.tx_hash).await {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(tx_hash = %req.tx_hash, error = %e, "leg resolution failed");
            return err(StatusCode::UNPROCESSABLE_ENTITY, e.to_string(), "UNRESOLVABLE_TX_HASH");
        }
    };

    tracing::info!(tx_hash = %req.tx_hash, legs = addresses.len(), "analyzing concurrently");
    let legs = join_all(addresses.iter().map(|a| adapter.analyze_leg(a))).await;
    let route_verdict = RouteReport::compute_verdict(&legs);

    // MEV risk on tx-hash path — heuristic based on legs, no value context
    let mev_risk = Some(mev::assess(&legs, None));

    tracing::info!(tx_hash = %req.tx_hash, verdict = ?route_verdict, "done");

    (StatusCode::OK, Json(ApiResponse::Ok(RouteReport {
        tx_hash: Some(req.tx_hash),
        chain: req.chain,
        route_verdict,
        legs,
        mev_risk,
        intent_resolution: None,
    })))
}

// ── POST /analyze-intent — pre-sign intent path ───────────────────────────

#[derive(Deserialize)]
pub struct IntentRequest {
    // Accept either a plain string or a structured object — serde handles both
    // via untagged enum. The intent module normalizes both forms via Claude.
    pub intent: serde_json::Value,
    pub chain: Option<String>,
}

pub async fn analyze_intent(
    State(state): State<SharedState>,
    Json(req): Json<IntentRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let api_key = match &state.anthropic_key {
        Some(k) => k.clone(),
        None => return err(StatusCode::SERVICE_UNAVAILABLE,
            "ANTHROPIC_API_KEY not configured on this instance", "LLM_UNAVAILABLE"),
    };

    // Convert intent value to string for the normalizer
    let raw_input = match &req.intent {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    let chain = req.chain.as_deref().unwrap_or("ethereum").to_string();

    let adapter = match state.adapters.get(&chain) {
        Some(a) => a,
        None => return err(StatusCode::BAD_REQUEST,
            format!("Chain '{chain}' not configured"), "UNSUPPORTED_CHAIN"),
    };

    tracing::info!(chain = %chain, "resolving intent");

    let (addresses, resolution) = match adapter.resolve_intent(&raw_input, &api_key).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "intent resolution failed");
            return err(StatusCode::UNPROCESSABLE_ENTITY, e.to_string(), "INTENT_UNPARSEABLE");
        }
    };

    tracing::info!(addresses = addresses.len(), "analyzing intent legs concurrently");
    let legs = join_all(addresses.iter().map(|a| adapter.analyze_leg(a))).await;
    let route_verdict = RouteReport::compute_verdict(&legs);

    // MEV risk on intent path — include value context if provided
    let mev_risk = Some(mev::assess(&legs, None));

    tracing::info!(verdict = ?route_verdict, "intent analysis done");

    (StatusCode::OK, Json(ApiResponse::Ok(RouteReport {
        tx_hash: None,
        chain,
        route_verdict,
        legs,
        mev_risk,
        intent_resolution: Some(resolution),
    })))
}
