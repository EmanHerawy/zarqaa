#!/usr/bin/env bash
# =============================================================================
# Zarqa Full-Stack Startup Script
#
# Starts the entire stack in order:
#   1. Zarqa Gateway (Rust)
#   2. AXL Node A  (Zarqa service node — owns the gateway + MCP router)
#   3. MCP Router  (Python — bridges AXL mesh calls to Zarqa HTTP)
#   4. Register zarqa service with the MCP router
#   5. AXL Node B  (OpenClaw trading agent node)
#   6. OpenClaw    (AI trading agent inside Docker)
#
# Run from the repo root:  ./start.sh
# Stop everything:         ./stop.sh
# =============================================================================

set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
LOG_DIR="$REPO_ROOT/logs"
mkdir -p "$LOG_DIR"

AXL_BIN="$REPO_ROOT/third_party/axl/bin/axl-node"
NODE_A_CFG="$REPO_ROOT/third_party/axl/configs/node-a.json"
NODE_B_CFG="$REPO_ROOT/third_party/axl/configs/node-b.json"
MCP_ROUTER_DIR="$REPO_ROOT/third_party/axl/integrations/mcp_routing"
ZARQA_CORE="$REPO_ROOT/zarqaa-core"
AGENT_FRAMEWORK="$REPO_ROOT/agent_framework"

# ── Ports ──────────────────────────────────────────────────────────────────
ZARQA_GW_PORT=8080
MCP_ROUTER_PORT=9003
NODE_A_API_PORT=9002
NODE_B_API_PORT=9012

# ── Helpers ────────────────────────────────────────────────────────────────
log() { echo "[$(date +%H:%M:%S)] $*"; }
wait_for_port() {
  local name=$1 port=$2 retries=${3:-30}
  log "Waiting for $name on port $port..."
  for i in $(seq 1 $retries); do
    if curl -sf "http://127.0.0.1:$port" >/dev/null 2>&1 || \
       curl -sf "http://127.0.0.1:$port/health" >/dev/null 2>&1 || \
       nc -z 127.0.0.1 "$port" 2>/dev/null; then
      log "$name is up."
      return 0
    fi
    sleep 1
  done
  log "WARNING: $name did not respond on port $port after ${retries}s — continuing anyway."
}

# ── 1. Zarqa Gateway (Rust) ────────────────────────────────────────────────
log "Starting Zarqa Gateway (Rust) on :$ZARQA_GW_PORT ..."
(
  cd "$ZARQA_CORE"
  cargo run -p zarqaa-gateway 2>&1
) > "$LOG_DIR/zarqa-gateway.log" 2>&1 &
ZARQA_GW_PID=$!
echo $ZARQA_GW_PID > "$LOG_DIR/zarqa-gateway.pid"
log "Zarqa Gateway PID=$ZARQA_GW_PID — logs: $LOG_DIR/zarqa-gateway.log"
wait_for_port "Zarqa Gateway" $ZARQA_GW_PORT 60

# ── 2. AXL Node A ─────────────────────────────────────────────────────────
log "Starting AXL Node A (Zarqa service node) — API :$NODE_A_API_PORT, P2P :7100 ..."
"$AXL_BIN" -config "$NODE_A_CFG" \
  > "$LOG_DIR/axl-node-a.log" 2>&1 &
NODE_A_PID=$!
echo $NODE_A_PID > "$LOG_DIR/axl-node-a.pid"
log "AXL Node A PID=$NODE_A_PID — logs: $LOG_DIR/axl-node-a.log"
wait_for_port "AXL Node A API" $NODE_A_API_PORT 30

# ── 3. MCP Router ─────────────────────────────────────────────────────────
log "Starting MCP Router on :$MCP_ROUTER_PORT ..."
(
  cd "$MCP_ROUTER_DIR"
  python3 mcp_router.py --port $MCP_ROUTER_PORT 2>&1
) > "$LOG_DIR/mcp-router.log" 2>&1 &
MCP_PID=$!
echo $MCP_PID > "$LOG_DIR/mcp-router.pid"
log "MCP Router PID=$MCP_PID — logs: $LOG_DIR/mcp-router.log"
wait_for_port "MCP Router" $MCP_ROUTER_PORT 20

# ── 4. Register zarqa service ─────────────────────────────────────────────
log "Registering zarqa service with MCP Router..."
curl -sf -X POST "http://127.0.0.1:$MCP_ROUTER_PORT/register" \
  -H "Content-Type: application/json" \
  -d "{\"service\":\"zarqa\",\"endpoint\":\"http://127.0.0.1:$ZARQA_GW_PORT/mcp\"}" \
  && log "zarqa service registered." \
  || log "WARNING: Failed to register zarqa — check MCP router logs."

# ── 5. AXL Node B ─────────────────────────────────────────────────────────
log "Starting AXL Node B (OpenClaw agent node) — API :$NODE_B_API_PORT ..."
"$AXL_BIN" -config "$NODE_B_CFG" \
  > "$LOG_DIR/axl-node-b.log" 2>&1 &
NODE_B_PID=$!
echo $NODE_B_PID > "$LOG_DIR/axl-node-b.pid"
log "AXL Node B PID=$NODE_B_PID — logs: $LOG_DIR/axl-node-b.log"
wait_for_port "AXL Node B API" $NODE_B_API_PORT 20

# Give nodes a moment to complete peering
sleep 2
log "AXL mesh status:"
curl -s "http://127.0.0.1:$NODE_B_API_PORT/topology" | python3 -c "
import sys, json
try:
    t = json.load(sys.stdin)
    peers = t.get('peers', [])
    for p in peers:
        status = 'UP' if p.get('up') else 'DOWN'
        print(f\"  Peer {p.get('public_key','?')[:16]}... [{status}]\")
    if not peers:
        print('  (no peers listed)')
except Exception as e:
    print(f'  (could not parse topology: {e})')
" 2>/dev/null || true

# ── 6. OpenClaw (Docker) ──────────────────────────────────────────────────
log "Starting OpenClaw agent (Docker)..."
(
  cd "$AGENT_FRAMEWORK"
  docker compose up -d --force-recreate openclaw-gateway 2>&1
)
log "OpenClaw started. Gateway: http://127.0.0.1:18789"

# ── Summary ───────────────────────────────────────────────────────────────
cat <<EOF

=== Stack is up ===

  Zarqa Gateway   http://127.0.0.1:8080          (PID $ZARQA_GW_PID)
  MCP Router      http://127.0.0.1:9003          (PID $MCP_PID)
  AXL Node A      http://127.0.0.1:9002          (PID $NODE_A_PID)
  AXL Node B      http://127.0.0.1:9012          (PID $NODE_B_PID)
  OpenClaw        http://127.0.0.1:18789

Test the end-to-end flow (from host):
  curl -s -X POST http://127.0.0.1:9012/mcp/5bcb2a071ed712d23bd97ce15b1aec892dd195fa18e67b061c22c2edfa45fd85/zarqa \\
    -H 'Content-Type: application/json' \\
    -d '{"jsonrpc":"2.0","method":"tools/call","id":1,"params":{"name":"zarqa_analyze_intent","arguments":{"intent":"swap 1 ETH for USDC on Uniswap V3","chain":"ethereum"}}}'

Test from OpenClaw workspace (inside Docker):
  docker exec zarqa-openclaw-gateway \\
    python3 /home/node/.openclaw/workspace/safety_guard.py \\
    intent "swap 1 ETH for USDC on Uniswap V3"

Logs: $LOG_DIR/
EOF
