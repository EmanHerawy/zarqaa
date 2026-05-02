#!/usr/bin/env bash
# Zarqa agent framework setup — non-interactive.
# Generates openclaw.json from .env and starts the gateway.
# No source checkout required — pulls ghcr.io/openclaw/openclaw:latest.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=============================================="
echo "  Zarqa Agent Framework — OpenClaw Setup"
echo "=============================================="
echo ""

# ── Prerequisites ─────────────────────────────────────────────────────────────
if ! docker info >/dev/null 2>&1; then
  echo "ERROR: Docker is not running. Start Docker Desktop and retry."
  exit 1
fi
echo "[ok] Docker is running"

# ── Bootstrap .env ────────────────────────────────────────────────────────────
if [ ! -f .env ]; then
  cp .env.example .env
  echo ""
  echo "[!!] .env created from template."
  echo "     Edit .env and set at least ANTHROPIC_API_KEY (or another provider key),"
  echo "     then re-run ./docker-setup.sh"
  echo ""
  exit 0
fi

set -a; source .env; set +a

# ── Defaults ──────────────────────────────────────────────────────────────────
OPENCLAW_MODEL="${OPENCLAW_MODEL:-anthropic/claude-sonnet-4-6}"
OPENCLAW_GATEWAY_PORT="${OPENCLAW_GATEWAY_PORT:-18789}"
TELEGRAM_DM_POLICY="${TELEGRAM_DM_POLICY:-open}"
DISCORD_DM_POLICY="${DISCORD_DM_POLICY:-open}"
WHATSAPP_DM_POLICY="${WHATSAPP_DM_POLICY:-open}"

# ── Validate provider key ─────────────────────────────────────────────────────
MODEL_PROVIDER=$(echo "$OPENCLAW_MODEL" | cut -d'/' -f1)
case "$MODEL_PROVIDER" in
  anthropic)
    [ -z "${ANTHROPIC_API_KEY:-}" ] && { echo "ERROR: ANTHROPIC_API_KEY is required for $OPENCLAW_MODEL"; exit 1; }
    echo "[ok] Provider: Anthropic ($OPENCLAW_MODEL)"
    ;;
  openai)
    [ -z "${OPENAI_API_KEY:-}" ] && { echo "ERROR: OPENAI_API_KEY is required for $OPENCLAW_MODEL"; exit 1; }
    echo "[ok] Provider: OpenAI ($OPENCLAW_MODEL)"
    ;;
  google)
    [ -z "${GEMINI_API_KEY:-}" ] && { echo "ERROR: GEMINI_API_KEY is required for $OPENCLAW_MODEL"; exit 1; }
    echo "[ok] Provider: Google ($OPENCLAW_MODEL)"
    ;;
  ollama)
    OLLAMA_HOST="${OLLAMA_HOST:-http://host.docker.internal:11434}"
    echo "[ok] Provider: Ollama ($OPENCLAW_MODEL) @ $OLLAMA_HOST"
    ;;
  *)
    echo "[!!] Unknown provider '$MODEL_PROVIDER' — ensure the right API key is set"
    ;;
esac

# ── Generate gateway token ────────────────────────────────────────────────────
if [ -z "${OPENCLAW_GATEWAY_TOKEN:-}" ]; then
  OPENCLAW_GATEWAY_TOKEN=$(openssl rand -hex 32)
  if grep -q "^OPENCLAW_GATEWAY_TOKEN=" .env; then
    sed -i.bak "s/^OPENCLAW_GATEWAY_TOKEN=.*/OPENCLAW_GATEWAY_TOKEN=$OPENCLAW_GATEWAY_TOKEN/" .env && rm -f .env.bak
  else
    echo "OPENCLAW_GATEWAY_TOKEN=$OPENCLAW_GATEWAY_TOKEN" >> .env
  fi
  echo "[ok] Generated gateway token"
fi

# ── Build channels JSON ───────────────────────────────────────────────────────
CHANNELS=""
PLUGINS=""

if [ -n "${TELEGRAM_BOT_TOKEN:-}" ]; then
  TG_ALLOW='["*"]'
  [ "$TELEGRAM_DM_POLICY" != "open" ] && TG_ALLOW='[]'
  CHANNELS="${CHANNELS:+$CHANNELS,
    }\"telegram\": {
      \"dmPolicy\": \"$TELEGRAM_DM_POLICY\",
      \"botToken\": \"$TELEGRAM_BOT_TOKEN\",
      \"allowFrom\": $TG_ALLOW,
      \"groupPolicy\": \"allowlist\",
      \"streamMode\": \"partial\"
    }"
  PLUGINS="${PLUGINS:+$PLUGINS,
      }\"telegram\": { \"enabled\": true }"
  echo "[ok] Telegram channel"
else
  echo "[--] Telegram: skipped (no TELEGRAM_BOT_TOKEN)"
fi

if [ -n "${DISCORD_BOT_TOKEN:-}" ]; then
  DC_ALLOW='["*"]'
  [ "$DISCORD_DM_POLICY" != "open" ] && DC_ALLOW='[]'
  CHANNELS="${CHANNELS:+$CHANNELS,
    }\"discord\": {
      \"token\": \"$DISCORD_BOT_TOKEN\",
      \"dm\": { \"enabled\": true, \"policy\": \"$DISCORD_DM_POLICY\", \"allowFrom\": $DC_ALLOW }
    }"
  PLUGINS="${PLUGINS:+$PLUGINS,
      }\"discord\": { \"enabled\": true }"
  echo "[ok] Discord channel"
else
  echo "[--] Discord: skipped (no DISCORD_BOT_TOKEN)"
fi

if [ -n "${WHATSAPP_ALLOW_FROM:-}" ]; then
  WA_ALLOW=$(echo "$WHATSAPP_ALLOW_FROM" | sed 's/,/","/g; s/^/["/; s/$/"]/')
  CHANNELS="${CHANNELS:+$CHANNELS,
    }\"whatsapp\": { \"dmPolicy\": \"$WHATSAPP_DM_POLICY\", \"allowFrom\": $WA_ALLOW }"
  echo "[ok] WhatsApp channel"
else
  echo "[--] WhatsApp: skipped (pair later with: docker compose run --rm --profile cli openclaw-cli channels login)"
fi

# ── Build optional providers block (Ollama) ───────────────────────────────────
MODELS_BLOCK=""
if [ "$MODEL_PROVIDER" = "ollama" ]; then
  OLLAMA_MODEL_NAME=$(echo "$OPENCLAW_MODEL" | cut -d'/' -f2-)
  MODELS_BLOCK="\"models\": {
    \"providers\": {
      \"ollama\": {
        \"api\": \"openai-completions\",
        \"baseUrl\": \"${OLLAMA_HOST}/v1\",
        \"apiKey\": \"ollama\",
        \"models\": [{\"id\": \"$OLLAMA_MODEL_NAME\", \"name\": \"$OLLAMA_MODEL_NAME\"}]
      }
    }
  },"
fi

# ── Generate .openclaw/openclaw.json ─────────────────────────────────────────
echo "[..] Generating openclaw.json..."
CONFIG_DIR="$SCRIPT_DIR/.openclaw"
mkdir -p "$CONFIG_DIR/workspace/memory" "$CONFIG_DIR/canvas" "$CONFIG_DIR/cron"

cat > "$CONFIG_DIR/openclaw.json" <<JSONEOF
{
  "logging": {
    "level": "info",
    "consoleLevel": "info",
    "consoleStyle": "pretty",
    "redactSensitive": "tools"
  },
  $MODELS_BLOCK
  "agents": {
    "defaults": {
      "model": {
        "primary": "$OPENCLAW_MODEL"
      },
      "workspace": "/home/node/.openclaw/workspace",
      "timeoutSeconds": 120,
      "maxConcurrent": 1,
      "subagents": {
        "maxConcurrent": 8
      }
    },
    "list": [
      {
        "id": "zarqa",
        "identity": {
          "name": "Zarqa",
          "theme": "Transaction-time security intelligence for Web3. Inspects every contract a transaction will touch and returns a per-component security report."
        }
      }
    ]
  },
  "tools": {
    "profile": "full"
  },
  "commands": {
    "native": "auto",
    "nativeSkills": "auto"
  },
  "session": {
    "scope": "per-sender",
    "resetTriggers": ["/new", "/reset"],
    "reset": {
      "mode": "idle",
      "idleMinutes": 30
    }
  },
  "channels": {
    $CHANNELS
  },
  "gateway": {
    "port": $OPENCLAW_GATEWAY_PORT,
    "mode": "local",
    "auth": {
      "token": "$OPENCLAW_GATEWAY_TOKEN"
    }
  },
  "messages": {
    "ackReactionScope": "group-mentions"
  },
  "plugins": {
    "entries": {
      $PLUGINS
    }
  }
}
JSONEOF

echo "[ok] openclaw.json written"

# ── Write mcporter config ─────────────────────────────────────────────────────
# mcporter connects MCP servers to OpenClaw via stdio inside the gateway container.
# The zarqa MCP server section below is commented out until the server is built.
MCPORTER_DIR="$SCRIPT_DIR/.mcporter"
mkdir -p "$MCPORTER_DIR"
cp "$SCRIPT_DIR/mcp-config.json" "$MCPORTER_DIR/mcporter.json"
echo "[ok] mcporter config written"

chmod -R 777 "$CONFIG_DIR" "$MCPORTER_DIR"

# ── Build openclaw:local if not already built ─────────────────────────────────
# The GHCR image ships with empty stubs; we must build from source.
OPENCLAW_SOURCE_DIR="${OPENCLAW_SOURCE_DIR:-$HOME/work/openclaw}"
if ! docker image inspect openclaw:local >/dev/null 2>&1; then
  if [ ! -d "$OPENCLAW_SOURCE_DIR" ]; then
    echo "[..] Cloning openclaw source to $OPENCLAW_SOURCE_DIR..."
    git clone https://github.com/openclaw/openclaw.git "$OPENCLAW_SOURCE_DIR"
  fi
  echo "[..] Building openclaw:local image (this takes ~5 min on first run)..."
  DOCKER_BUILDKIT=1 docker build \
    -t openclaw:local \
    -f "$OPENCLAW_SOURCE_DIR/Dockerfile" \
    "$OPENCLAW_SOURCE_DIR"
  echo "[ok] openclaw:local built"
else
  echo "[ok] openclaw:local image already exists"
fi

echo "[..] Starting gateway..."
docker compose up -d openclaw-gateway

echo ""
echo "=============================================="
echo "  Setup complete"
echo "=============================================="
echo ""
echo "  Control UI : http://localhost:$OPENCLAW_GATEWAY_PORT"
echo "  Token      : ${OPENCLAW_GATEWAY_TOKEN:0:16}..."
echo "  Model      : $OPENCLAW_MODEL"
echo ""
echo "Channels:"
[ -n "${TELEGRAM_BOT_TOKEN:-}" ] && echo "  - Telegram : enabled"
[ -n "${DISCORD_BOT_TOKEN:-}" ]  && echo "  - Discord  : enabled"
[ -z "${TELEGRAM_BOT_TOKEN:-}" ] && [ -z "${DISCORD_BOT_TOKEN:-}" ] && \
  echo "  (none — add bot tokens to .env and re-run)"
echo ""
echo "Commands:"
echo "  Logs   : docker compose logs -f openclaw-gateway"
echo "  Stop   : docker compose down"
echo "  Health : curl http://localhost:$OPENCLAW_GATEWAY_PORT/healthz"
echo "  WhatsApp pair: docker compose run --rm --profile cli openclaw-cli channels login"
echo "=============================================="
