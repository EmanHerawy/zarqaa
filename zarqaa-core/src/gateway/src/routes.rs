use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use zarqaa_types::report::RouteReport;

use crate::state::SharedState;

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub tx_hash: String,
    pub chain: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum AnalyzeResponse {
    Ok(RouteReport),
    Err(ApiError),
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}

pub async fn analyze(
    State(state): State<SharedState>,
    Json(req): Json<AnalyzeRequest>,
) -> (StatusCode, Json<AnalyzeResponse>) {
    let adapter = match state.adapters.get(&req.chain) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(AnalyzeResponse::Err(ApiError {
                    error: format!("Chain '{}' is not configured on this instance", req.chain),
                    code: "UNSUPPORTED_CHAIN".to_string(),
                })),
            );
        }
    };

    tracing::info!(tx_hash = %req.tx_hash, chain = %req.chain, "analyzing transaction");

    let addresses = match adapter.resolve_legs(&req.tx_hash).await {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(tx_hash = %req.tx_hash, error = %e, "leg resolution failed");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(AnalyzeResponse::Err(ApiError {
                    error: e.to_string(),
                    code: "UNRESOLVABLE_TX_HASH".to_string(),
                })),
            );
        }
    };

    let mut legs = Vec::new();
    for addr in &addresses {
        legs.push(adapter.analyze_leg(addr).await);
    }

    let route_verdict = RouteReport::compute_verdict(&legs);

    tracing::info!(
        tx_hash = %req.tx_hash,
        legs = legs.len(),
        verdict = ?route_verdict,
        "analysis complete"
    );

    let report = RouteReport {
        tx_hash: req.tx_hash,
        chain: req.chain,
        route_verdict,
        legs,
    };

    (StatusCode::OK, Json(AnalyzeResponse::Ok(report)))
}
