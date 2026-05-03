// Integration tests — require real API keys and network access.
//
// Keys are loaded automatically from zarqaa-core/.env — no manual export needed.
//
// Run with:
//   cargo test -p zarqaa-adapters -- --ignored
//
// These tests hit real mainnet data. The tx hashes used are from confirmed
// blocks and will never change, so the assertions are stable.

use std::sync::OnceLock;
use zarqaa_adapters::{EvmAdapter, EvmChainConfig};
use zarqaa_types::report::Verdict;

// OnceLock ensures dotenvy is called exactly once even when tests run in parallel.
// Without this, two tests could race on std::env::set_var which is not thread-safe.
static ENV: OnceLock<()> = OnceLock::new();

fn setup() {
    ENV.get_or_init(|| { dotenvy::dotenv().ok(); });
}

fn adapter() -> EvmAdapter {
    setup();
    EvmAdapter::new(EvmChainConfig::ethereum_from_env())
}

// ── REAL TX HASH — replace this with one from Etherscan ──────────────────────
// Go to https://etherscan.io/address/0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45
// Click any recent transaction → copy the tx hash → paste below.
// The router address is Uniswap V3 SwapRouter02. Any swap through it touches
// multiple contracts (router, pool, 2 token contracts) which is what we need.
const UNISWAP_V3_SWAP_TX: &str = "0xda1a5adb70e061cbaf129250228f2161f783083bff9cd8fef5514f0b103de6b4";

// ── End-to-end: known Uniswap V3 swap ────────────────────────────────────────
#[tokio::test]
#[ignore]
async fn e2e_uniswap_v3_swap_all_legs_verified() {
    assert_ne!(UNISWAP_V3_SWAP_TX, "REPLACE_WITH_REAL_TX_HASH",
        "Set UNISWAP_V3_SWAP_TX to a real tx hash from Etherscan");

    let a = adapter();
    let addresses = a.resolve_legs(UNISWAP_V3_SWAP_TX).await
        .expect("resolve_legs should succeed for a confirmed mainnet tx");

    assert!(!addresses.is_empty(), "a swap touches at least one contract");

    let mut legs = Vec::new();
    for addr in &addresses {
        legs.push(a.analyze_leg(addr).await);
    }

    for leg in &legs {
        assert_ne!(
            leg.verdict, Verdict::Red,
            "leg {} should not be Red — notes: {:?}", leg.address, leg.notes
        );
    }

    let verified_count = legs.iter().filter(|l| l.source_verified).count();
    assert!(verified_count > 0,
        "at least one contract should be source-verified — notes per leg: {:#?}",
        legs.iter().map(|l| &l.notes).collect::<Vec<_>>()
    );
}

// ── Resolve legs: tx with multiple inner contract calls ───────────────────────
#[tokio::test]
#[ignore]
async fn resolve_legs_returns_multiple_addresses_for_swap() {
    assert_ne!(UNISWAP_V3_SWAP_TX, "REPLACE_WITH_REAL_TX_HASH",
        "Set UNISWAP_V3_SWAP_TX to a real tx hash from Etherscan");

    let addresses = adapter().resolve_legs(UNISWAP_V3_SWAP_TX).await
        .expect("should resolve");
    assert!(addresses.len() >= 2,
        "expected multiple legs, got {}. Addresses: {:?}", addresses.len(), addresses);
}

// ── Source check: Uniswap V3 Router is verified ───────────────────────────────
#[tokio::test]
#[ignore]
async fn uniswap_v3_router_is_source_verified() {
    // SwapRouter02 — deployed, immutable, always verified on Etherscan
    let router = "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45";
    let leg = adapter().analyze_leg(router).await;

    // If this fails, check leg.notes — it will show the actual error
    // (e.g. "Source check failed: Explorer rate limited" means the API key isn't loaded)
    assert!(leg.source_verified,
        "Uniswap V3 router should be source verified — notes: {:?}", leg.notes);
    assert!(!leg.is_proxy,
        "Uniswap V3 router is not a proxy — notes: {:?}", leg.notes);
}

// ── Proxy detection: Uniswap V3 factory is NOT a proxy ───────────────────────
#[tokio::test]
#[ignore]
async fn non_proxy_contract_returns_false() {
    let factory = "0x1f98431c8ad98523631ae4a59f267346ea31f984";
    let leg = adapter().analyze_leg(factory).await;
    assert!(!leg.is_proxy, "Uniswap factory is not a proxy — notes: {:?}", leg.notes);
    assert!(leg.proxy_implementation.is_none());
}

// ── Known infra: LayerZero V2 endpoint detected ───────────────────────────────
#[tokio::test]
#[ignore]
async fn layerzero_v2_endpoint_detected_as_infra() {
    let lz_endpoint = "0x1a44076050125825900e736c501f859c50fe728c";
    let leg = adapter().analyze_leg(lz_endpoint).await;

    assert!(leg.infra_kind.is_some(),
        "LayerZero endpoint should be detected as infra — notes: {:?}", leg.notes);
    assert!(leg.infra_kind.as_deref().unwrap().contains("LayerZero"),
        "infra label should mention LayerZero, got: {:?}", leg.infra_kind);
    assert_ne!(leg.verdict, Verdict::Green,
        "known infra should not be Green — notes: {:?}", leg.notes);
}

// ── Error handling: non-existent tx hash ─────────────────────────────────────
#[tokio::test]
#[ignore]
async fn resolve_legs_errors_on_unknown_tx() {
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
