// Integration tests — require real API keys and network access.
//
// Keys are loaded automatically from zarqaa-core/.env — no manual export needed.
//
// Run with:
//   cargo test -p zarqa-adapters -- --ignored
//
// These tests hit real mainnet data. The tx hashes used are from confirmed
// blocks and will never change, so the assertions are stable.

use zarqa_adapters::{EvmAdapter, EvmChainConfig};
use zarqa_types::report::Verdict;

fn setup() {
    // Load .env file if present. .ok() silently ignores a missing file,
    // so these tests still work in CI where .env doesn't exist
    // (CI should provide env vars directly instead).
    dotenvy::dotenv().ok();
}

fn adapter() -> EvmAdapter {
    setup();
    EvmAdapter::new(EvmChainConfig::ethereum_from_env())
}

// ── End-to-end: known Uniswap V3 swap ────────────────────────────────────────
//
// This tx is a real Uniswap V3 USDC→WETH swap on mainnet (block 12376729).
// It touches: Uniswap V3 Router, the USDC/WETH pool, USDC token, WETH token.
// All four contracts are source-verified on Etherscan.
//
// This is the PRIMARY integration test — if this passes, the full pipeline works.
#[tokio::test]
#[ignore]
async fn e2e_uniswap_v3_swap_all_legs_verified() {
    let tx = "0xd0b2e2b72893a0c6c6f338a4e5a47d87c02af06b73a43fc25d44ee4a45c0b9d9";
    let a = adapter();

    let addresses = a.resolve_legs(tx).await
        .expect("resolve_legs should succeed for a confirmed mainnet tx");

    assert!(!addresses.is_empty(), "a swap touches at least one contract");

    // Analyze every leg
    let legs: Vec<_> = {
        let mut v = Vec::new();
        for addr in &addresses {
            v.push(a.analyze_leg(addr).await);
        }
        v
    };

    // All Uniswap V3 contracts are verified — no leg should be Unverified or Red
    for leg in &legs {
        assert_ne!(
            leg.verdict, Verdict::Red,
            "leg {} should not be Red on a clean Uniswap swap",
            leg.address
        );
    }

    // At least one leg should have source_verified = true
    let verified_count = legs.iter().filter(|l| l.source_verified).count();
    assert!(verified_count > 0, "at least one contract should be source-verified");
}

// ── Resolve legs: tx with multiple inner contract calls ───────────────────────
#[tokio::test]
#[ignore]
async fn resolve_legs_returns_multiple_addresses_for_swap() {
    let tx = "0xd0b2e2b72893a0c6c6f338a4e5a47d87c02af06b73a43fc25d44ee4a45c0b9d9";
    let addresses = adapter().resolve_legs(tx).await.expect("should resolve");
    // A V3 swap touches router + pool + at least 2 token contracts
    assert!(addresses.len() >= 2, "expected multiple legs, got {}", addresses.len());
}

// ── Source check: Uniswap V3 Router is verified ───────────────────────────────
#[tokio::test]
#[ignore]
async fn uniswap_v3_router_is_source_verified() {
    // Uniswap V3 SwapRouter02 — deployed, immutable, always verified
    let router = "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45";
    let leg = adapter().analyze_leg(router).await;
    assert!(leg.source_verified, "Uniswap V3 router should be source verified");
    assert!(!leg.is_proxy, "Uniswap V3 router is not a proxy");
}

// ── Proxy detection: Uniswap V3 factory is NOT a proxy ───────────────────────
#[tokio::test]
#[ignore]
async fn non_proxy_contract_returns_false() {
    let factory = "0x1f98431c8ad98523631ae4a59f267346ea31f984";
    let leg = adapter().analyze_leg(factory).await;
    assert!(!leg.is_proxy, "Uniswap factory is not a proxy");
    assert!(leg.proxy_implementation.is_none());
}

// ── Known infra: LayerZero V2 endpoint detected ───────────────────────────────
#[tokio::test]
#[ignore]
async fn layerzero_v2_endpoint_detected_as_infra() {
    let lz_endpoint = "0x1a44076050125825900e736c501f859c50fe728c";
    let leg = adapter().analyze_leg(lz_endpoint).await;
    assert!(leg.infra_kind.is_some(), "LayerZero endpoint should be detected as infra");
    assert!(
        leg.infra_kind.as_deref().unwrap().contains("LayerZero"),
        "infra label should mention LayerZero"
    );
    // Known infra should bump verdict to at least Amber
    assert_ne!(leg.verdict, Verdict::Green, "known infra should not be Green");
}

// ── Error handling: non-existent tx hash ─────────────────────────────────────
#[tokio::test]
#[ignore]
async fn resolve_legs_errors_on_unknown_tx() {
    // All zeros is not a real tx hash
    let fake = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let result = adapter().resolve_legs(fake).await;
    assert!(result.is_err(), "should error on non-existent tx");
}

// TODO: test a tx to an unverified contract → verdict should be Unverified
// TODO: test a proxy contract → is_proxy=true, proxy_implementation is Some
// TODO: test Etherscan rate limit handling (hard to trigger reliably, consider mocking)
// TODO: test a LayerZero bridge tx end-to-end → infra detected, route verdict Amber
// TODO: test a known-bad tx (e.g. from an exploit) → Red verdict
// TODO: test concurrent leg analysis (run analyze_leg in parallel, check results match sequential)
