use axum::{extract::State, http::StatusCode, Json};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use zarqaa_adapters::mev;
use zarqaa_types::report::{RouteReport, Verdict};

use crate::state::SharedState;

// ── Shared REST response envelope ─────────────────────────────────────────

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

// ── Core analysis helpers (shared by REST + MCP) ──────────────────────────

async fn run_tx_analysis(state: &SharedState, tx_hash: &str, chain: &str)
    -> Result<RouteReport, (StatusCode, String, String)>
{
    let adapter = state.adapters.get(chain)
        .ok_or_else(|| (StatusCode::BAD_REQUEST,
            format!("Chain '{chain}' not configured"), "UNSUPPORTED_CHAIN".into()))?;

    let addresses = adapter.resolve_legs(tx_hash).await
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string(), "UNRESOLVABLE_TX_HASH".into()))?;

    let legs = join_all(addresses.iter().map(|a| adapter.analyze_leg(a))).await;
    let route_verdict = RouteReport::compute_verdict(&legs);
    let mev_risk = Some(mev::assess(&legs, None));

    Ok(RouteReport { tx_hash: Some(tx_hash.to_string()), chain: chain.to_string(), route_verdict, legs, mev_risk, intent_resolution: None })
}

async fn run_intent_analysis(state: &SharedState, intent: &str, chain: &str)
    -> Result<RouteReport, (StatusCode, String, String)>
{
    let llm = state.llm.as_ref()
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE,
            "LLM_API_KEY not configured on this instance".into(), "LLM_UNAVAILABLE".into()))?;

    let adapter = state.adapters.get(chain)
        .ok_or_else(|| (StatusCode::BAD_REQUEST,
            format!("Chain '{chain}' not configured"), "UNSUPPORTED_CHAIN".into()))?;

    let (addresses, resolution) = adapter.resolve_intent(intent, llm).await
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string(), "INTENT_UNPARSEABLE".into()))?;

    let legs = join_all(addresses.iter().map(|a| adapter.analyze_leg(a))).await;
    let route_verdict = RouteReport::compute_verdict(&legs);
    let mev_risk = Some(mev::assess(&legs, None));

    Ok(RouteReport { tx_hash: None, chain: chain.to_string(), route_verdict, legs, mev_risk, intent_resolution: Some(resolution) })
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
    tracing::info!(tx_hash = %req.tx_hash, chain = %req.chain, "analyze request");
    match run_tx_analysis(&state, &req.tx_hash, &req.chain).await {
        Ok(report) => (StatusCode::OK, Json(ApiResponse::Ok(report))),
        Err((status, msg, code)) => err(status, msg, &code),
    }
}

// ── POST /analyze-intent — pre-sign intent path ───────────────────────────

#[derive(Deserialize)]
pub struct IntentRequest {
    pub intent: serde_json::Value,
    pub chain: Option<String>,
}

pub async fn analyze_intent(
    State(state): State<SharedState>,
    Json(req): Json<IntentRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let raw_input = match &req.intent {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let chain = req.chain.as_deref().unwrap_or("ethereum").to_string();

    tracing::info!(chain = %chain, "analyze-intent request");
    match run_intent_analysis(&state, &raw_input, &chain).await {
        Ok(report) => (StatusCode::OK, Json(ApiResponse::Ok(report))),
        Err((status, msg, code)) => err(status, msg, &code),
    }
}

// ── POST /mcp — MCP JSON-RPC endpoint (used by AXL mesh) ─────────────────
//
// Exposes two tools over the Model Context Protocol so any AXL peer can call
// the Zarqa security pipeline without knowing about the REST API directly.

#[derive(Deserialize)]
pub struct McpRequest {
    pub jsonrpc: serde_json::Value,
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

pub async fn mcp(
    State(state): State<SharedState>,
    Json(req): Json<McpRequest>,
) -> Json<serde_json::Value> {
    let id = req.id.clone();
    match req.method.as_str() {
        "tools/list" => Json(mcp_tools_list(id)),
        "tools/call" => Json(mcp_tools_call(state, id, req.params).await),
        _ => Json(mcp_err(id, -32601, "Method not found")),
    }
}

fn mcp_tools_list(id: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "zarqa_analyze_transaction",
                    "description": "Analyze a submitted Ethereum transaction hash for security risks. Returns per-contract verdicts (Green/Amber/Red/Unverified), MEV risk level, bridge security cards, and an overall route verdict.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tx_hash": {"type": "string", "description": "0x-prefixed transaction hash"},
                            "chain":   {"type": "string", "description": "Chain name (default: ethereum)"}
                        },
                        "required": ["tx_hash"]
                    }
                },
                {
                    "name": "zarqa_analyze_intent",
                    "description": "Analyze a pre-sign transaction intent or natural language description BEFORE signing. Use this whenever a user is about to send a transaction or asks if a route is safe (e.g. 'swap 1 ETH for USDC on Uniswap V3'). Returns per-contract security report.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "intent": {"type": "string", "description": "Intent string, e.g. 'swap 1 ETH for USDC on Uniswap V3 on Ethereum' or raw tx JSON"},
                            "chain":  {"type": "string", "description": "Chain name (default: ethereum)"}
                        },
                        "required": ["intent"]
                    }
                }
            ]
        }
    })
}

async fn mcp_tools_call(
    state: SharedState,
    id: serde_json::Value,
    params: Option<serde_json::Value>,
) -> serde_json::Value {
    let params = match params {
        Some(p) => p,
        None => return mcp_err(id, -32602, "Missing params"),
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return mcp_err(id, -32602, "Missing params.name"),
    };

    let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    let result: Result<RouteReport, (StatusCode, String, String)> = match tool_name.as_str() {
        "zarqa_analyze_transaction" => {
            let tx_hash = match args.get("tx_hash").and_then(|v| v.as_str()) {
                Some(h) => h.to_string(),
                None => return mcp_err(id, -32602, "Missing argument: tx_hash"),
            };
            let chain = args.get("chain").and_then(|v| v.as_str()).unwrap_or("ethereum").to_string();
            tracing::info!(tool = "zarqa_analyze_transaction", tx_hash = %tx_hash, chain = %chain, "MCP call");
            run_tx_analysis(&state, &tx_hash, &chain).await
        }
        "zarqa_analyze_intent" => {
            let intent = match args.get("intent").and_then(|v| v.as_str()) {
                Some(i) => i.to_string(),
                None => return mcp_err(id, -32602, "Missing argument: intent"),
            };
            let chain = args.get("chain").and_then(|v| v.as_str()).unwrap_or("ethereum").to_string();
            tracing::info!(tool = "zarqa_analyze_intent", chain = %chain, "MCP call");
            run_intent_analysis(&state, &intent, &chain).await
        }
        _ => return mcp_err(id, -32602, &format!("Unknown tool: {tool_name}")),
    };

    match result {
        Ok(report) => {
            let verdict_str = match &report.route_verdict {
                Verdict::Green      => "Green",
                Verdict::Amber      => "Amber",
                Verdict::Red        => "Red",
                Verdict::Unverified => "Unverified",
            };
            let mev_level = report.mev_risk.as_ref()
                .map(|m| format!("{:?}", m.risk_level))
                .unwrap_or_else(|| "None".into());
            let mev_rec = report.mev_risk.as_ref()
                .map(|m| m.recommendation.clone())
                .unwrap_or_default();

            let flagged: Vec<_> = report.legs.iter()
                .filter(|l| !matches!(l.verdict, Verdict::Green))
                .collect();
            let summary = if flagged.is_empty() {
                format!("All {} contract(s) verified and clean.", report.legs.len())
            } else {
                let issues: Vec<String> = flagged.iter().take(3).map(|l| {
                    let note = l.notes.first().cloned().unwrap_or_else(|| format!("{:?}", l.verdict));
                    format!("{}…: {}", &l.address[..10.min(l.address.len())], note)
                }).collect();
                format!("{} of {} contract(s) flagged — {}", flagged.len(), report.legs.len(), issues.join("; "))
            };

            let content_text = serde_json::json!({
                "verdict":          verdict_str,
                "summary":          summary,
                "mev_risk_level":   mev_level,
                "mev_recommendation": mev_rec,
                "full_report":      serde_json::to_value(&report).unwrap_or_default()
            });

            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{"type": "text", "text": content_text.to_string()}]
                }
            })
        }
        Err((_status, msg, code)) => mcp_err(id, -32000, &format!("[{code}] {msg}")),
    }
}

fn mcp_err(id: serde_json::Value, code: i32, message: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}
