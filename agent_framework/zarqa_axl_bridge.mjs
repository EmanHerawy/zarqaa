#!/usr/bin/env node
/**
 * Zarqa AXL Bridge — stdio MCP server for OpenClaw/mcporter.
 *
 * Receives MCP JSON-RPC calls from OpenClaw via stdin,
 * routes them through AXL Node B → mesh → Node A → Zarqa Gateway,
 * writes the response back to stdout.
 *
 * Required env vars:
 *   ZARQA_AXL_URL        — Node B's AXL HTTP API (default: http://host.docker.internal:9012)
 *   ZARQA_NODE_A_PEER_ID — Node A's 64-char hex public key
 */

import { createInterface } from 'readline';

const AXL_URL    = process.env.ZARQA_AXL_URL       || 'http://host.docker.internal:9012';
const NODE_A_ID  = process.env.ZARQA_NODE_A_PEER_ID || '';
const SERVICE    = 'zarqa';
const TIMEOUT_MS = 90_000;

if (!NODE_A_ID) {
  process.stderr.write('[zarqa-bridge] WARNING: ZARQA_NODE_A_PEER_ID not set — calls will fail\n');
}

// ── Tool definitions (mirrors the Rust gateway) ───────────────────────────

const TOOLS = [
  {
    name: 'zarqa_analyze_transaction',
    description:
      'Analyze a submitted Ethereum transaction hash for security risks. ' +
      'Returns per-contract verdicts (Green/Amber/Red/Unverified), MEV risk, ' +
      'bridge security cards, and an overall route verdict.',
    inputSchema: {
      type: 'object',
      properties: {
        tx_hash: { type: 'string', description: '0x-prefixed transaction hash' },
        chain:   { type: 'string', description: 'Chain name (default: ethereum)' },
      },
      required: ['tx_hash'],
    },
  },
  {
    name: 'zarqa_analyze_intent',
    description:
      'Analyze a pre-sign transaction intent BEFORE signing. ' +
      'Call this whenever a user is about to send a transaction or asks if a ' +
      "route is safe (e.g. 'swap 1 ETH for USDC on Uniswap V3'). " +
      'Also use when the user asks about the security of any DeFi operation.',
    inputSchema: {
      type: 'object',
      properties: {
        intent: { type: 'string', description: "Intent, e.g. 'swap 1 ETH for USDC on Uniswap V3'" },
        chain:  { type: 'string', description: 'Chain name (default: ethereum)' },
      },
      required: ['intent'],
    },
  },
];

// ── MCP request handler ────────────────────────────────────────────────────

async function handle(req) {
  const { jsonrpc, id, method, params } = req;

  if (method === 'initialize') {
    return { jsonrpc, id, result: { protocolVersion: '2024-11-05', capabilities: { tools: {} }, serverInfo: { name: 'zarqa', version: '1.0.0' } } };
  }

  if (method === 'tools/list') {
    return { jsonrpc, id, result: { tools: TOOLS } };
  }

  if (method === 'tools/call') {
    const toolName = params?.name;
    const args     = params?.arguments ?? {};
    if (!toolName) return rpcErr(id, -32602, 'Missing params.name');

    try {
      const result = await callViaAXL(toolName, args);
      return { jsonrpc, id, result: { content: [{ type: 'text', text: JSON.stringify(result) }] } };
    } catch (e) {
      process.stderr.write(`[zarqa-bridge] tool call failed: ${e.message}\n`);
      return rpcErr(id, -32000, e.message);
    }
  }

  // Ignore notifications (no id), unknown methods return error
  if (id === undefined || id === null) return null;
  return rpcErr(id, -32601, `Method not found: ${method}`);
}

// ── AXL mesh call ──────────────────────────────────────────────────────────

async function callViaAXL(toolName, args) {
  if (!NODE_A_ID) throw new Error('ZARQA_NODE_A_PEER_ID not configured');

  const url  = `${AXL_URL}/mcp/${NODE_A_ID}/${SERVICE}`;
  const body = JSON.stringify({ jsonrpc: '2.0', method: 'tools/call', id: 1, params: { name: toolName, arguments: args } });

  process.stderr.write(`[zarqa-bridge] → ${url} tool=${toolName}\n`);

  const resp = await fetch(url, {
    method:  'POST',
    headers: { 'Content-Type': 'application/json' },
    body,
    signal:  AbortSignal.timeout(TIMEOUT_MS),
  });

  if (!resp.ok) {
    const text = await resp.text().catch(() => '');
    throw new Error(`AXL returned ${resp.status}: ${text}`);
  }

  const data = await resp.json();

  if (data.error) throw new Error(`Zarqa error: ${JSON.stringify(data.error)}`);

  const text = data?.result?.content?.[0]?.text;
  if (!text) throw new Error('Empty response from Zarqa service');

  return JSON.parse(text);
}

function rpcErr(id, code, message) {
  return { jsonrpc: '2.0', id, error: { code, message } };
}

// ── Stdio transport ────────────────────────────────────────────────────────

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });

rl.on('line', async (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;

  let req;
  try { req = JSON.parse(trimmed); } catch {
    process.stdout.write(JSON.stringify(rpcErr(null, -32700, 'Parse error')) + '\n');
    return;
  }

  const response = await handle(req);
  if (response !== null) {
    process.stdout.write(JSON.stringify(response) + '\n');
  }
});

rl.on('close', () => process.exit(0));
process.stderr.write('[zarqa-bridge] ready\n');
