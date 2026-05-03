use std::collections::HashMap;
use zarqaa_types::report::BridgeInfo;

// Returns a human-readable label if the address is a known bridge or oracle.
// Keyed by (chain_id, address_lowercase).
//
// Why hardcoded? For an MVP, ~30 addresses covers all major infra.
// Zero latency, zero network dependency. Phase 2 complements with
// on-chain enumeration (e.g. LayerZero getUlnConfig).
pub fn known_infra_label(chain_id: &str, address: &str) -> Option<String> {
    let addr = address.to_lowercase();
    infra_map().get(&(chain_id, addr.as_str())).map(|e| e.label.to_string())
}

// Returns mocked bridge security data for known infrastructure.
// is_mocked: true on all entries — Phase 2 replaces with live on-chain reads.
//
// TODO (Phase 2): read DVN config from LayerZero UlnV2 on-chain,
//   read Wormhole guardian set count from core contract,
//   read CCIP ARM/RMN config from ARMProxy contract.
pub fn bridge_mock_info(chain_id: &str, address: &str) -> Option<BridgeInfo> {
    let addr = address.to_lowercase();
    infra_map().get(&(chain_id, addr.as_str())).and_then(|e| e.bridge_info.clone())
}

struct InfraEntry {
    label: &'static str,
    bridge_info: Option<BridgeInfo>,
}

fn ccip_info(destination_chain: Option<&str>) -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "Chainlink CCIP".to_string(),
        dvn_count: Some(5),
        relayer_type: "don".to_string(),
        required_confirmations: Some(12),
        centralization_risk: "low".to_string(),
        last_incident: None,
        destination_chain: destination_chain.map(str::to_string),
        is_mocked: true,
    })
}

fn layerzero_v1_info() -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "LayerZero V1".to_string(),
        dvn_count: Some(1),
        relayer_type: "contract".to_string(),
        required_confirmations: Some(15),
        centralization_risk: "medium".to_string(),
        last_incident: None,
        destination_chain: None,
        is_mocked: true,
    })
}

fn layerzero_v2_info() -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "LayerZero V2".to_string(),
        dvn_count: Some(1),
        relayer_type: "contract".to_string(),
        required_confirmations: Some(15),
        // Single DVN by default — apps can configure more but most don't
        centralization_risk: "medium".to_string(),
        last_incident: None,
        destination_chain: None,
        is_mocked: true,
    })
}

fn wormhole_info() -> Option<BridgeInfo> {
    Some(BridgeInfo {
        protocol: "Wormhole".to_string(),
        dvn_count: Some(19),
        relayer_type: "contract".to_string(),
        required_confirmations: Some(15),
        centralization_risk: "low".to_string(),
        last_incident: Some("2022-02-02: Wormhole exploit ($320M, ETH side minted without collateral)".to_string()),
        destination_chain: None,
        is_mocked: true,
    })
}

fn e(label: &'static str, bridge_info: Option<BridgeInfo>) -> InfraEntry {
    InfraEntry { label, bridge_info }
}

fn infra_map() -> HashMap<(&'static str, &'static str), InfraEntry> {
    let mut map = HashMap::new();

    // ── Chainlink CCIP ───────────────────────────────────────────────────────
    // Ethereum mainnet — destination is typically Arbitrum or Base for most flows
    map.insert(("ethereum", "0x80226fc0ee2b096224eeac085bb9a8cba1146f7d"),
        e("Chainlink CCIP Router",        ccip_info(Some("arbitrum"))));
    map.insert(("ethereum", "0x411de17f12d1a34ecc7f45f49844626267c75e81"),
        e("Chainlink CCIP ARMProxy",      ccip_info(None)));
    map.insert(("ethereum", "0xe8464c353210cc398a45db2454fbc5bcd25fff20"),
        e("Chainlink CCIP RMNRemote",     ccip_info(None)));
    map.insert(("ethereum", "0x913814782144864e523c3fdb78e3ca25d2c2aeca"),
        e("Chainlink CCIP OnRamp",        ccip_info(Some("arbitrum"))));
    map.insert(("ethereum", "0x300f2ca3e3867133baea866c89096f097d57bf57"),
        e("Chainlink CCIP FeeQuoter",     ccip_info(None)));
    map.insert(("ethereum", "0xb22764f98dd05c789929716d677382df22c05cb6"),
        e("Chainlink CCIP TokenAdminRegistry", ccip_info(None)));

    // ── LayerZero V1 ─────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x66a71dcef29a0ffbdbe3c6a460a3b5bc225cd675"),
        e("LayerZero V1 Endpoint", layerzero_v1_info()));
    map.insert(("arbitrum", "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1_info()));
    map.insert(("optimism", "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1_info()));
    map.insert(("polygon",  "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1_info()));
    map.insert(("bsc",      "0x3c2269811836af69497e5f486a85d7316753cf62"),
        e("LayerZero V1 Endpoint", layerzero_v1_info()));

    // ── LayerZero V2 ─────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2_info()));
    map.insert(("arbitrum", "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2_info()));
    map.insert(("optimism", "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2_info()));
    map.insert(("base",     "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2_info()));
    map.insert(("polygon",  "0x1a44076050125825900e736c501f859c50fe728c"),
        e("LayerZero V2 EndpointV2", layerzero_v2_info()));

    // ── Wormhole ─────────────────────────────────────────────────────────────
    map.insert(("ethereum", "0x98f3c9e6e3face36baad05fe09d375ef1464288b"),
        e("Wormhole Core Bridge", wormhole_info()));
    map.insert(("bsc",      "0x98f3c9e6e3face36baad05fe09d375ef1464288b"),
        e("Wormhole Core Bridge", wormhole_info()));

    // ── Chainlink Price Feeds (oracle, no bridge_info) ───────────────────────
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

    // ── Pyth Network ────────────────────────────────────────────────────────
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
    fn ccip_router_has_bridge_info() {
        let info = bridge_mock_info("ethereum", "0x80226fc0ee2b096224eeac085bb9a8cba1146f7d")
            .expect("CCIP Router should have bridge info");
        assert_eq!(info.protocol, "Chainlink CCIP");
        assert_eq!(info.centralization_risk, "low");
        assert_eq!(info.destination_chain.as_deref(), Some("arbitrum"));
        assert!(info.is_mocked);
    }

    #[test]
    fn layerzero_v2_has_medium_risk() {
        let info = bridge_mock_info("ethereum", "0x1a44076050125825900e736c501f859c50fe728c")
            .expect("LayerZero V2 should have bridge info");
        assert_eq!(info.centralization_risk, "medium");
        assert_eq!(info.dvn_count, Some(1));
    }

    #[test]
    fn wormhole_has_incident_history() {
        let info = bridge_mock_info("ethereum", "0x98f3c9e6e3face36baad05fe09d375ef1464288b")
            .expect("Wormhole should have bridge info");
        assert!(info.last_incident.is_some());
        assert_eq!(info.dvn_count, Some(19));
    }

    #[test]
    fn chainlink_price_feed_has_no_bridge_info() {
        let info = bridge_mock_info("ethereum", "0x5f4ec3df9cbd43714fe2740f5e3616155c5b8419");
        assert!(info.is_none(), "price feeds are not bridges");
    }
}
