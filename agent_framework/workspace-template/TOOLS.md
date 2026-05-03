# TOOLS.md - Local Notes

## AXL Mesh Network

### Node A (Zarqaa Service)
- **Public Key:** `5bcb2a071ed712d23bd97ce15b1aec892dd195fa18e67b061c22c2edfa45fd85`
- **IPv6:** `201:90d3:57e3:84a3:b4b7:109a:c7a:9394`
- **HTTP API:** `http://127.0.0.1:9002`
- **P2P Port:** `7100`
- **MCP Router:** `http://127.0.0.1:9003`
- **Zarqaa Gateway:** `http://127.0.0.1:8080`
- **MCP endpoint:** `http://127.0.0.1:8080/mcp`

### Node B (This Agent)
- **Public Key:** `d1c50ffa17bb8ea243c5a03f2c18a235dc6a5252e95ac9a9df11c9a5785ae175`
- **IPv6:** `200:5c75:e00b:d088:e2bb:7874:bf81:a7ce`
- **HTTP API:** `http://127.0.0.1:9012`
- **P2P Port:** `7101`

## Safety Guard

Run a security check from the workspace:
```bash
python3 safety_guard.py intent "swap 1 ETH for USDC on Uniswap V3"
python3 safety_guard.py tx_hash "0xabc..."
```

Exit codes: 0 = Green, 1 = Amber/Unverified, 2 = Red

## Node A Startup Checklist

1. Start Zarqa gateway: `cd zarqaa-core && cargo run -p zarqaa-gateway`
2. Start MCP router: `python -m mcp_routing.mcp_router --port 9003`
3. Register zarqa service:
   ```bash
   curl -X POST http://localhost:9003/register \
     -H "Content-Type: application/json" \
     -d '{"service":"zarqa","endpoint":"http://127.0.0.1:8080/mcp"}'
   ```
4. Start Node A: `./bin/axl-node -config configs/node-a.json`

## Mesh Status

⚠️ Node B peer shows `up: false` — confirm Node A is running and Node B can reach port 7000.
Check: `curl http://127.0.0.1:9012/topology`
