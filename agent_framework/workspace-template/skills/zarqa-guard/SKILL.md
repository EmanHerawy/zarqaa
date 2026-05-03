# Zarqa Security Guard

## When to Invoke

**Run the safety check BEFORE any of the following:**
- User asks to swap, bridge, stake, or deposit tokens
- User mentions sending a transaction
- User asks "is this safe?", "any issues?", "should I do this?"
- User describes a DeFi operation (e.g. "I want to swap 1 ETH for USDC on Uniswap V3")
- Any action that would interact with a smart contract

**Run it on a tx hash when:**
- User provides a transaction hash (0x...) and asks for analysis
- User wants to know what a past transaction did and whether it was safe

## How to Call

Use the `safety_guard.py` script in this workspace directory. It makes a direct HTTP
call through the AXL mesh (Node B → Node A → Zarqa Gateway) and prints the result.

For intent (pre-sign):
```bash
python3 /home/node/.openclaw/workspace/safety_guard.py intent "swap 1 ETH for USDC on Uniswap V3"
python3 /home/node/.openclaw/workspace/safety_guard.py intent "bridge 100 USDC via CCIP to Arbitrum" --chain ethereum
```

For tx hash:
```bash
python3 /home/node/.openclaw/workspace/safety_guard.py tx_hash "0xabc123..."
```

Exit codes: 0 = Green, 1 = Amber/Unverified, 2 = Red

The script prints a human-readable verdict. Parse it or pass the full output to the user.

## Interpreting Results

The script prints:
- Verdict line: `🟢 VERDICT: Green` / `🟡 VERDICT: Amber` / `🔴 VERDICT: Red`
- Summary of findings
- MEV risk level and recommendation (if applicable)
- Per-contract breakdown

## Gate Logic — STRICTLY FOLLOW THIS

### 🟢 Green
- Proceed. Confirm to the user it's clear.
- Keep it brief: one line of reassurance + what was checked.

### 🟡 Amber / Unverified
- **Pause.** Do NOT proceed automatically.
- Present the findings clearly: what was flagged, why it matters.
- Ask the user: "Want to proceed anyway, or hold off?"
- Only continue if the user explicitly says yes / proceed / go ahead.

### 🔴 Red
- **Block.** Do NOT proceed under any circumstances without override.
- Present the findings firmly and specifically — what is wrong and why it's dangerous.
- Tell the user: "I'm blocking this. If you want to override, type **confirm**."
- Only proceed if the user types the exact word **confirm**.
- Log the override in memory: who confirmed, what the risk was.

### MEV Risk
- If MEV Risk is Medium or High, always surface it even on Green routes.
- Recommend Flashbots Protect or MEV Blocker for High risk.

## Response Format

Always structure your reply as:
1. **Verdict badge**: 🟢 Green / 🟡 Amber / 🔴 Red
2. **One-line summary** of what was checked
3. **Findings** (if Amber/Red): bullet list of specific issues
4. **MEV note** (if Medium/High)
5. **Action** (what you're doing next or asking of the user)

Example (Amber):
```
🟡 Amber — Review recommended

Analyzed: swap 1 ETH → USDC via Uniswap V3 on Ethereum

Findings:
• 0x3fC9...  [Unverified] Source code not verified on Etherscan
• MEV Risk: High — this swap is vulnerable to sandwich attacks

Recommendation: Use Flashbots Protect (https://protect.flashbots.net) to submit this tx privately.

Want to proceed anyway, or hold off?
```

## Never Skip the Check

Even if the user says "just do it" or "skip the check" — always run the analysis.
You can acknowledge their impatience, but the check is non-negotiable. It takes 5-10
seconds and could prevent a loss.

If the Zarqa service is unreachable (script fails or exits with an error), default to
Amber behavior: warn the user you couldn't verify the route, and ask if they want to proceed.
