use std::collections::HashMap;
use zarqaa_types::report::BridgeInfo;

pub fn known_infra_label(chain_id: &str, address: &str) -> Option<String> {
    let addr = address.to_lowercase();
    infra_map().get(&(chain_id, addr.as_str())).map(|e| e.label.to_string())
}

// Returns the full Bridge Security Card for known infrastructure.
// All fields marked is_mocked=true until Phase 2 on-chain reads.
//
// TODO (Phase 2):
//   - CCIP: read ARM/RMN contract for real risk config + rate limits
//   - LayerZero: call getUlnConfig(oapp, eid) for real DVN list per route
//   - Wormhole: call getCurrentGuardianSetIndex() + getGuardianSet() on core
//   - Recent flags: integrate Rekt.news + DefiHackLabs feed (30-day window)
//   - Timelock: read delay() from detected timelock contract on-chain
pub fn bridge_mock_info(chain_id: &str, address: &str) -> Option<BridgeInfo> {
    let addr = address.to_lowercase();
    infra_map().get(&(chain_id, addr.as_str())).and_then(|e| e.bridge_info.clone())
}

struct InfraEntry {
    label: &'static str,
    bridge_info: Option<BridgeInfo>,
}

fn e(label: &'static str, bridge_info: Option<BridgeInfo>) -> InfraEntry {
    InfraEntry { label, bridge_info }
}

fn ccip(destination_chain: Option<&str>) -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "Chainlink CCIP".to_string(),
        summary: "Double-checked by two independent teams".to_string(),

        dvn_count: Some(5),
        controller_type: "don".to_string(),
        upgrade_timelock_days: Some(7),
        relayer_type: "don".to_string(),

        past_exploits_usd: Some(0),
        past_exploit_note: None,
        last_audit: Some("Continuous (multiple firms)".to_string()),
        bug_bounty_usd: None,

        has_rate_limits: true,
        has_circuit_breaker: true,
        emergency_pause_by: Some("Independent Risk Management Network".to_string()),

        recent_flags: vec![],

        verdict_label: "Proceed".to_string(),
        verdict_summary: "Multiple independent checks make this the most robust option. \
                          Slower than others, but safer.".to_string(),

        centralization_risk: "low".to_string(),
        destination_chain: destination_chain.map(str::to_string),
        is_mocked: true,
    })
}

fn layerzero_v1() -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "LayerZero V1".to_string(),
        summary: "You choose your own guards — choose wisely".to_string(),

        dvn_count: Some(1),
        controller_type: "multisig".to_string(),
        upgrade_timelock_days: Some(2),
        relayer_type: "contract".to_string(),

        past_exploits_usd: Some(0),
        past_exploit_note: Some(
            "$293M lost via misconfigured app using 1-of-1 DVN (Radiant Capital, Oct 2024)"
                .to_string(),
        ),
        last_audit: Some("Multiple firms".to_string()),
        bug_bounty_usd: Some(15_000_000),

        has_rate_limits: true,
        has_circuit_breaker: false,
        emergency_pause_by: Some("App owner only".to_string()),

        recent_flags: vec![],

        verdict_label: "Review".to_string(),
        verdict_summary: "Core protocol is solid but apps must configure multiple DVN verifiers. \
                          Verify your app's DVN setup before proceeding.".to_string(),

        centralization_risk: "medium".to_string(),
        destination_chain: None,
        is_mocked: true,
    })
}

fn layerzero_v2() -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "LayerZero V2".to_string(),
        summary: "You choose your own guards — choose wisely".to_string(),

        dvn_count: Some(1),
        controller_type: "multisig".to_string(),
        upgrade_timelock_days: Some(2),
        relayer_type: "contract".to_string(),

        past_exploits_usd: Some(0),
        past_exploit_note: Some(
            "$293M lost via misconfigured app using 1-of-1 DVN (Radiant Capital, Oct 2024)"
                .to_string(),
        ),
        last_audit: Some("Jan 2026 (Hexens) — updated Mar 2026".to_string()),
        bug_bounty_usd: Some(15_000_000),

        has_rate_limits: true,
        has_circuit_breaker: false,
        emergency_pause_by: Some("App owner only".to_string()),

        recent_flags: vec![],

        verdict_label: "Review".to_string(),
        verdict_summary: "Core protocol is solid but apps must configure multiple DVN verifiers. \
                          Verify your app's DVN setup before proceeding.".to_string(),

        centralization_risk: "medium".to_string(),
        destination_chain: None,
        is_mocked: true,
    })
}

fn wormhole() -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "Wormhole".to_string(),
        summary: "19 known guardians — 13 must agree to move your funds".to_string(),

        dvn_count: Some(19),
        controller_type: "guardian_council".to_string(),
        upgrade_timelock_days: Some(3),
        relayer_type: "contract".to_string(),

        past_exploits_usd: Some(326_000_000),
        past_exploit_note: Some(
            "Feb 2022 — $326M. Fully repaid by Jump Crypto. Root cause patched.".to_string(),
        ),
        last_audit: Some("Ongoing (29 total audits across multiple firms)".to_string()),
        bug_bounty_usd: Some(10_000_000),

        has_rate_limits: true,
        has_circuit_breaker: true,
        emergency_pause_by: Some("Guardian supermajority (13-of-19)".to_string()),

        recent_flags: vec![],

        verdict_label: "Proceed with Awareness".to_string(),
        verdict_summary: "Strong protections now, but history shows guardian compromise is \
                          possible. Suitable for medium-sized transfers.".to_string(),

        centralization_risk: "low".to_string(),
        destination_chain: None,
        is_mocked: true,
    })
}

fn infra_map() -> HashMap<(&'static str, &'static str), InfraEntry> {
    let mut map = HashMap::new();

    // ── Chainlink CCIP (Ethereum mainnet) ────────────────────────────────────
    map.insert(("ethereum", "0x80226fc0ee2b096224eeac085bb9a8cba1146f7d"),
        e("Chainlink CCIP Router",             ccip(Some("arbitrum"))));
    map.insert(("ethereum", "0x411de17f12d1a34ecc7f45f49844626267c75e81"),
        e("Chainlink CCIP ARMProxy",           ccip(None)));
    map.insert(("ethereum", "0xe8464c353210cc398a45db2454fbc5bcd25fff20"),
        e("Chainlink CCIP RMNRemote",          ccip(None)));
    map.insert(("ethereum", "0x913814782144864e523c3fdb78e3ca25d2c2aeca"),
        e("Chainlink CCIP OnRamp",             ccip(Some("arbitrum"))));
    map.insert(("ethereum", "0x300f2ca3e3867133baea866c89096f097d57bf57"),
        e("Chainlink CCIP FeeQuoter",          ccip(None)));
    map.insert(("ethereum", "0xb22764f98dd05c789929716d677382df22c05cb6"),
        e("Chainlink CCIP TokenAdminRegistry", ccip(None)));

    // ── LayerZero V1 ─────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x66a71dcef29a0ffbdbe3c6a460a3b5bc225cd675"),
        e("LayerZero V1 Endpoint", layerzero_v1()));
    map.insert(("arbitrum", "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1()));
    map.insert(("optimism", "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1()));
    map.insert(("polygon",  "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1()));
    map.insert(("bsc",      "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1()));

    // ── LayerZero V2 ─────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2()));
    map.insert(("arbitrum", "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2()));
    map.insert(("optimism", "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2()));
    map.insert(("base",     "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2()));
    map.insert(("polygon",  "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2()));

    // ── Wormhole ─────────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x98f3c9e6e3face36baad05fe09d375ef1464288b"),
        e("Wormhole Core Bridge", wormhole()));
    map.insert(("bsc",      "0x98f3c9e6e3face36baad05fe09d375ef1464288b"),
        e("Wormhole Core Bridge", wormhole()));

    // ── Chainlink Price Feeds ────────────────────────────────────────────────
    map.insert(("ethereum", "0x5f4ec3df9cbd43714fe2740f5e3616155c5b8419"),
        e("Chainlink ETH/USD Feed", None));
    map.insert(("arbitrum", "0x639fe6ab55c921f74e7fac1ee960c0b6293ba612"),
        e("Chainlink ETH/USD Feed", None));
    map.insert(("optimism", "0x13e3ee699d1909e989722e753853ae30b17e08c5"),
        e("Chainlink ETH/USD Feed", None));
    map.insert(("base",     "0x71041dddad3595f9ced3dccfbe3d1f4b0a16bb70"),
        e("Chainlink ETH/USD Feed", None));
    map.insert(("polygon",  "0xf9680d99d6c9589e2a93a78a04a279e509205945"),
        e("Chainlink ETH/USD Feed", None));

    // ── Pyth Network ─────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x4305fb66699c3b2702d4d05cf36551390a4c69c6"),
        e("Pyth Network Oracle", None));
    map.insert(("arbitrum", "0xff1a0f4744e8582df1ae09d5611b887b6a12925c"),
        e("Pyth Network Oracle", None));

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layerzero_v2_detected_on_ethereum() {
        let label = known_infra_label("ethereum", "0x1a44076050125825900e736c501f859c50fe728c");
        assert_eq!(label.as_deref(), Some("LayerZero V2 EndpointV2"));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let label = known_infra_label("ethereum", "0x1A44076050125825900E736C501f859c50fE728c");
        assert_eq!(label.as_deref(), Some("LayerZero V2 EndpointV2"));
    }

    #[test]
    fn unknown_address_returns_none() {
        let label = known_infra_label("ethereum", "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef");
        assert!(label.is_none());
    }

    #[test]
    fn wrong_chain_returns_none() {
        let label = known_infra_label("solana", "0x1a44076050125825900e736c501f859c50fe728c");
        assert!(label.is_none());
    }

    #[test]
    fn chainlink_feed_detected_on_arbitrum() {
        let label = known_infra_label("arbitrum", "0x639fe6ab55c921f74e7fac1ee960c0b6293ba612");
        assert_eq!(label.as_deref(), Some("Chainlink ETH/USD Feed"));
    }

    #[test]
    fn ccip_router_verdict_is_proceed() {
        let info = bridge_mock_info("ethereum", "0x80226fc0ee2b096224eeac085bb9a8cba1146f7d")
            .expect("CCIP Router should have bridge info");
        assert_eq!(info.verdict_label, "Proceed");
        assert_eq!(info.centralization_risk, "low");
        assert_eq!(info.destination_chain.as_deref(), Some("arbitrum"));
        assert!(info.has_circuit_breaker);
        assert!(info.is_mocked);
    }

    #[test]
    fn layerzero_v2_verdict_is_review() {
        let info = bridge_mock_info("ethereum", "0x1a44076050125825900e736c501f859c50fe728c")
            .expect("LayerZero V2 should have bridge info");
        assert_eq!(info.verdict_label, "Review");
        assert_eq!(info.dvn_count, Some(1));
        assert!(!info.has_circuit_breaker);
        assert!(info.past_exploit_note.is_some());
    }

    #[test]
    fn wormhole_has_exploit_history_and_proceed_verdict() {
        let info = bridge_mock_info("ethereum", "0x98f3c9e6e3face36baad05fe09d375ef1464288b")
            .expect("Wormhole should have bridge info");
        assert_eq!(info.past_exploits_usd, Some(326_000_000));
        assert!(info.past_exploit_note.is_some());
        assert!(info.verdict_label.starts_with("Proceed"));
        assert_eq!(info.dvn_count, Some(19));
        assert!(info.has_circuit_breaker);
    }

    #[test]
    fn chainlink_price_feed_has_no_bridge_info() {
        let info = bridge_mock_info("ethereum", "0x5f4ec3df9cbd43714fe2740f5e3616155c5b8419");
        assert!(info.is_none());
    }
}
