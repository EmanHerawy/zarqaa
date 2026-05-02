# Zarqa — زرقاء

> Zarqa sees the hidden risks. You decide whether to listen.

![Zarqa](docs/intro.png)
---

## The Story of Zarqa in history

In ancient Arabia, there was a woman named Zarqa al-Yamama — زرقاء اليمامة — of the Jadis tribe. She was known for eyesight so sharp she could see a rider from three days' distance. When an enemy army approached, camouflaged behind cut trees, she was the only one who saw them. She warned her tribe. They dismissed her.

The army arrived. The tribe fell.

We named this tool after her because she is the original early warning system — and because the story of being right, being ignored, and watching the damage happen is one that everyone in Web3 security knows by heart.

Zarqa brings that same foresight to Web3. It looks past the surface of a transaction to reveal the "hidden soldiers"—compromised proxies, unverified owners, and stale audits—before they can strike your assets.

---

## What is Zarqa

Zarqa is a transaction-time security intelligence layer for Web3.

It accepts a transaction in two states — one you are about to sign, or one that has already been submitted. In both cases it resolves the full execution path, inspects every contract your transaction will touch, and returns a per-component security report. Audit status. Ownership structure. Proxy upgrade history. Exploit reports. MEV exposure. Sourced, cited, and streamed to you in real time.

It answers one question:

> *"Is the infrastructure this transaction depends on trustworthy — and has any of it been compromised recently?"*

It does not tell you how much you will receive. It does not predict price. It does not replace your wallet. It shows you what is hidden inside the trees — before you sign, or after you've already moved.

---

## The gap it fills

Excellent security tools exist at the extremes. Static analyzers (Slither, Mythril) audit code before deployment. Forensic platforms (Chainalysis, TRM) trace funds after an exploit. Wallet simulators (Blowfish, Blockaid) catch phishing patterns at signing time.

Nothing aggregates trust signals — audit history, ownership structure, proxy upgrade status, recent exploit reports, MEV exposure — at the moment of decision, for the specific contracts your transaction will actually touch.

The canonical example: a user bridges 50,000 USDC via a LayerZero-based bridge. At signing time, no tool tells them that the bridge's DVN uses a single verifier, that verifier's RPC endpoint was reported compromised six days ago, the bridge contract's owner is a single EOA with no timelock, and the last audit predates the most recent proxy upgrade by four months.

All of this information exists on-chain or in public data sources. It is simply not aggregated at the point of decision.

That is the problem Zarqa is built to solve — and only this problem.
---

## The flight path model

Think of a DeFi transaction like a flight with multiple legs. A swap might route through an aggregator router, a liquidity pool, a price oracle, and a token contract. Each leg is a separate smart contract. Each one carries its own risk profile.

Zarqa resolves every leg, then runs a security check on each one:

- Is the source code publicly verified?
- When was it last audited — and does that audit post-date the last proxy upgrade?
- Who controls the contract — a single wallet, a multisig, a DAO?
- Has it appeared in an exploit report recently?
- Is this transaction exposed to MEV sandwich attacks?

The result is a per-leg security card — green, amber, red, or unverified — plus an overall route verdict.

**If Zarqa cannot assess a component for any reason, it says so explicitly with a reason code. Silence is never treated as safe.**

---

## How it works

Zarqa works in two ways. Whether you are inspecting a route before you execute it, or examining a transaction that has already been submitted or confirmed — the analysis pipeline is identical. Same depth, same data sources, same trust model, same output schema.

---


### Path 1 — Intent (pre-sign)

You have a route in mind. A swap, a bridge, a deposit. You have not signed yet.

Submit the sender address, target contract, and calldata — or a higher-level intent. Zarqa resolves every contract in the execution path and runs the full security check pipeline on each one.

**Every finding is prospective and actionable. You can still walk away.**

This is the highest-value flow. Use it to inspect a route before committing to it — to check whether the bridge you are about to use has a clean ownership structure, whether the liquidity pool has a recent audit, whether the oracle has been flagged in the last week.


**Who uses this:** DeFi traders, DAO treasury operators, protocol integrators, anyone who wants to know what they are signing before they sign it.

---

### Path 2 — Transaction hash

The transaction already exists — submitted to the mempool, or confirmed on-chain.

Submit the transaction hash. Zarqa resolves it via RPC, decodes the full execution trace, extracts the call graph, and runs the same analysis pipeline.


**Who uses this:** Security researchers investigating incidents, DAO treasuries reviewing past transactions, compliance teams building audit trails, anyone who received a transaction hash and wants to understand what it did.

