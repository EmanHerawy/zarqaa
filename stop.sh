#!/usr/bin/env bash
# Kill everything started by start.sh
REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
LOG_DIR="$REPO_ROOT/logs"

log() { echo "[$(date +%H:%M:%S)] $*"; }

# Stop OpenClaw
log "Stopping OpenClaw..."
(cd "$REPO_ROOT/agent_framework" && docker compose down) 2>/dev/null || true

# Kill PIDs recorded by start.sh
for pidfile in "$LOG_DIR"/*.pid; do
  [ -f "$pidfile" ] || continue
  pid=$(cat "$pidfile")
  name=$(basename "$pidfile" .pid)
  if kill -0 "$pid" 2>/dev/null; then
    log "Stopping $name (PID $pid)..."
    kill "$pid" 2>/dev/null || true
  fi
  rm -f "$pidfile"
done

log "All services stopped."
