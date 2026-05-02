# Zarqa Agent Framework

OpenClaw gateway running in Docker — the AI backbone Zarqa's analysis pipeline subagents connect to.

## Prerequisites

- [Docker Desktop](https://docs.docker.com/get-docker/) (Mac/Windows) or Docker Engine + Docker Compose v2 (Linux)
- At least 2 GB RAM available to Docker
- An Anthropic API key

## First-time setup

```bash
cd agent_framework
chmod +x docker-setup.sh
./docker-setup.sh
```

The script will:
1. Pull the pre-built OpenClaw image from GitHub Container Registry (no source checkout needed)
2. Prompt you for your Anthropic API key during onboarding
3. Generate a secure gateway token and write it to `.env`
4. Start the gateway on `http://127.0.0.1:18789`

Once running, open `http://127.0.0.1:18789` in your browser and paste the token shown in the terminal into Settings.

## Environment variables

Override defaults by setting these before running `docker-setup.sh`:

| Variable | Default | Purpose |
|---|---|---|
| `OPENCLAW_IMAGE` | `ghcr.io/openclaw/openclaw:latest` | Use a specific image tag |
| `OPENCLAW_CONFIG_DIR` | `~/.openclaw` | Where gateway config and auth are stored on the host |
| `OPENCLAW_WORKSPACE_DIR` | `~/.openclaw/workspace` | Agent workspace directory |
| `OPENCLAW_GATEWAY_PORT` | `18789` | Host port for the gateway and Control UI |
| `OPENCLAW_GATEWAY_TOKEN` | _(auto-generated)_ | Shared secret — set this to share a token with the team |

Example for a pinned image and shared token:

```bash
export OPENCLAW_IMAGE="ghcr.io/openclaw/openclaw:2026.4.29"
export OPENCLAW_GATEWAY_TOKEN="your-shared-token-here"
./docker-setup.sh
```

After first run, all values are persisted to `.env` so subsequent starts just use `docker compose up`.

## Day-to-day commands

Run these from the `agent_framework/` directory:

```bash
# Start the gateway (after first-time setup)
docker compose up -d openclaw-gateway

# Stop the gateway
docker compose down

# View live logs
docker compose logs -f openclaw-gateway

# Health check
curl -fsS http://127.0.0.1:18789/healthz

# Run a CLI command (e.g. list connected devices)
docker compose run --rm openclaw-cli devices list

# Add a Telegram channel
docker compose run --rm openclaw-cli channels add --channel telegram --token <token>

# Open the Control UI dashboard link
docker compose run --rm openclaw-cli dashboard --no-open
```

## Team setup

Everyone runs `./docker-setup.sh` on their own machine. Each person gets their own gateway instance with their own API keys. This is the recommended flow for development.

For a **shared team gateway** (e.g. on a VPS), set a common `OPENCLAW_GATEWAY_TOKEN` in `.env` and share it with the team. Each team member connects their Claude Code to the same host:port using that token.

## Zarqa extension

Zarqa subagents connect to the gateway via the MCP tool protocol. Each pipeline stage (path resolver, exploit scanner, etc.) is registered as an MCP tool the orchestrator calls. When adding a new Zarqa subagent:

1. Build it as an MCP server using the `Dockerfile.txt` template in this directory
2. Register it with the running gateway:
   ```bash
   docker compose run --rm openclaw-cli config set --batch-json \
     '[{"path":"mcp.servers.<name>.command","value":"<command>"}]'
   ```
3. The orchestrator (Mode A) will automatically discover and call it when building the task graph

See `docs/zarqa_subagent_mcp_map.svg` for the full subagent topology.

## Troubleshooting

**Docker daemon I/O error on macOS**
`input/output error` on `metadata.db` is a Docker Desktop bug. Fix: quit and restart Docker Desktop, then rerun the setup script.

**Permission errors on `/home/node/.openclaw`**
The container runs as `node` (uid 1000). If the config directory was created by root:
```bash
sudo chown -R 1000:1000 ~/.openclaw
```

**Port 18789 already in use**
```bash
export OPENCLAW_GATEWAY_PORT=18790
./docker-setup.sh
```

**OOM-killed during image pull (exit 137)**
Increase Docker Desktop memory to at least 2 GB in Settings → Resources.

**Re-run onboarding to change API keys**
```bash
docker compose run --rm --no-deps openclaw-gateway openclaw.mjs onboard --mode local --no-install-daemon
docker compose restart openclaw-gateway
```
