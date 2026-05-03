#!/usr/bin/env python3
"""
Zarqa Safety Guard — call the Zarqa security service via AXL mesh.

The agent (Node B) routes the request through the AXL mesh to Node A,
which runs the MCP router and Zarqa Gateway.

Usage (as a shell tool from the agent workspace):
  python3 safety_guard.py intent  "swap 1 ETH for USDC on Uniswap V3"
  python3 safety_guard.py tx_hash "0xabc123..."
  python3 safety_guard.py intent  "bridge 100 USDC via CCIP to Arbitrum" --chain ethereum

Reads peer config from TOOLS.md or env vars:
  ZARQA_AXL_URL        — Node B AXL API (default: http://127.0.0.1:9012)
  ZARQA_NODE_A_PEER_ID — Node A's 64-char hex public key
"""

import argparse, json, os, sys, re
import urllib.request, urllib.error

AXL_URL   = os.getenv("ZARQA_AXL_URL", "http://127.0.0.1:9012")
PEER_ID   = os.getenv("ZARQA_NODE_A_PEER_ID", "")
SERVICE   = "zarqa"
TIMEOUT   = 90  # seconds

def _load_peer_id_from_tools_md():
    """Fallback: read Node A peer ID from TOOLS.md in this workspace."""
    tools_path = os.path.join(os.path.dirname(__file__), "TOOLS.md")
    if not os.path.exists(tools_path):
        return ""
    with open(tools_path) as f:
        content = f.read()
    m = re.search(r"Node A.*?Public Key.*?`([0-9a-fA-F]{64})`", content, re.DOTALL)
    return m.group(1) if m else ""


def call_zarqa(tool_name: str, arguments: dict) -> dict:
    peer_id = PEER_ID or _load_peer_id_from_tools_md()
    if not peer_id:
        sys.exit("[zarqa] ERROR: ZARQA_NODE_A_PEER_ID not set. Add it to TOOLS.md or set the env var.")

    url  = f"{AXL_URL}/mcp/{peer_id}/{SERVICE}"
    body = json.dumps({
        "jsonrpc": "2.0",
        "method":  "tools/call",
        "id":      1,
        "params":  {"name": tool_name, "arguments": arguments},
    }).encode()

    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
            data = json.loads(resp.read())
    except urllib.error.HTTPError as e:
        sys.exit(f"[zarqa] HTTP {e.code}: {e.read().decode()}")
    except Exception as e:
        sys.exit(f"[zarqa] Request failed: {e}")

    if "error" in data:
        sys.exit(f"[zarqa] RPC error: {data['error']}")

    text = data.get("result", {}).get("content", [{}])[0].get("text", "")
    if not text:
        sys.exit("[zarqa] Empty response from Zarqa service")
    return json.loads(text)


def format_report(result: dict) -> str:
    verdict  = result.get("verdict", "Unknown")
    summary  = result.get("summary", "")
    mev_lvl  = result.get("mev_risk_level", "None")
    mev_rec  = result.get("mev_recommendation", "")

    icon = {"Green": "🟢", "Amber": "🟡", "Red": "🔴", "Unverified": "⚪"}.get(verdict, "❓")

    lines = [
        f"{icon} VERDICT: {verdict}",
        f"Summary: {summary}",
    ]
    if mev_lvl not in ("None", ""):
        lines.append(f"MEV Risk: {mev_lvl}")
        if mev_rec:
            lines.append(f"  → {mev_rec}")

    report = result.get("full_report", {})
    legs   = report.get("legs", [])
    if legs:
        lines.append(f"\nContracts inspected ({len(legs)}):")
        for leg in legs:
            leg_verdict = leg.get("verdict", "?")
            addr        = leg.get("address", "")
            notes       = leg.get("notes", [])
            note_str    = notes[0] if notes else ""
            leg_icon    = {"Green": "✓", "Amber": "⚠", "Red": "✗", "Unverified": "?"}.get(leg_verdict, "?")
            lines.append(f"  {leg_icon} {addr[:12]}…  [{leg_verdict}]  {note_str}")

        unresolved = report.get("intent_resolution", {})
        if unresolved:
            ur = unresolved.get("unresolved_legs", [])
            if ur:
                lines.append(f"\nDynamic legs (not statically resolved):")
                for u in ur:
                    lines.append(f"  • {u}")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Zarqa safety guard")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_intent = sub.add_parser("intent", help="Analyze a transaction intent before signing")
    p_intent.add_argument("intent", help="Natural language intent or raw tx JSON")
    p_intent.add_argument("--chain", default="ethereum")

    p_hash = sub.add_parser("tx_hash", help="Analyze a submitted transaction by hash")
    p_hash.add_argument("tx_hash", help="0x-prefixed transaction hash")
    p_hash.add_argument("--chain", default="ethereum")

    args = parser.parse_args()

    if args.cmd == "intent":
        result = call_zarqa("zarqa_analyze_intent", {"intent": args.intent, "chain": args.chain})
    else:
        result = call_zarqa("zarqa_analyze_transaction", {"tx_hash": args.tx_hash, "chain": args.chain})

    print(format_report(result))

    # Exit code encodes verdict for shell scripts
    verdict = result.get("verdict", "Unknown")
    sys.exit({"Green": 0, "Amber": 1, "Red": 2, "Unverified": 1}.get(verdict, 3))


if __name__ == "__main__":
    main()
