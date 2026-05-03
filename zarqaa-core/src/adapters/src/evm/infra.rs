use std::collections::HashMap;

// Returns a human-readable label if the address is a known bridge or oracle.
// The HashMap is keyed by (chain_id, address_lowercase).
//
// Why hardcoded? For an MVP, a curated list of ~20 addresses covers all the
// major infrastructure. It's zero latency and zero network dependency.
// Phase 2 would complement this with on-chain enumeration (LayerZero getUlnConfig).
pub fn known_infra_label(chain_id: &str, address: &str) -> Option<String> {
    let addr = address.to_lowercase();

    let mut map: HashMap<(&str, &str), &str> = HashMap::new();

    // LayerZero V1 Endpoint
    map.insert(("ethereum", "0x66a71dcef29a0ffbdbe3c6a460a3b5bc225cd675"), "LayerZero V1 Endpoint");
    map.insert(("arbitrum", "0x3c2269811836af69497e5f486a85d7316753cf62"), "LayerZero V1 Endpoint");
    map.insert(("optimism", "0x3c2269811836af69497e5f486a85d7316753cf62"), "LayerZero V1 Endpoint");
    map.insert(("polygon",  "0x3c2269811836af69497e5f486a85d7316753cf62"), "LayerZero V1 Endpoint");
    map.insert(("bsc",      "0x3c2269811836af69497e5f486a85d7316753cf62"), "LayerZero V1 Endpoint");

    // LayerZero V2 EndpointV2
    map.insert(("ethereum", "0x1a44076050125825900e736c501f859c50fe728c"), "LayerZero V2 EndpointV2");
    map.insert(("arbitrum", "0x1a44076050125825900e736c501f859c50fe728c"), "LayerZero V2 EndpointV2");
    map.insert(("optimism", "0x1a44076050125825900e736c501f859c50fe728c"), "LayerZero V2 EndpointV2");
    map.insert(("base",     "0x1a44076050125825900e736c501f859c50fe728c"), "LayerZero V2 EndpointV2");
    map.insert(("polygon",  "0x1a44076050125825900e736c501f859c50fe728c"), "LayerZero V2 EndpointV2");

    // Wormhole Core Bridge
    map.insert(("ethereum", "0x98f3c9e6e3face36baad05fe09d375ef1464288b"), "Wormhole Core Bridge");
    map.insert(("bsc",      "0x98f3c9e6e3face36baad05fe09d375ef1464288b"), "Wormhole Core Bridge");

    // Chainlink ETH/USD price feeds
    map.insert(("ethereum", "0x5f4ec3df9cbd43714fe2740f5e3616155c5b8419"), "Chainlink ETH/USD Feed");
    map.insert(("arbitrum", "0x639fe6ab55c921f74e7fac1ee960c0b6293ba612"), "Chainlink ETH/USD Feed");
    map.insert(("optimism", "0x13e3ee699d1909e989722e753853ae30b17e08c5"), "Chainlink ETH/USD Feed");
    map.insert(("base",     "0x71041dddad3595f9ced3dccfbe3d1f4b0a16bb70"), "Chainlink ETH/USD Feed");
    map.insert(("polygon",  "0xf9680d99d6c9589e2a93a78a04a279e509205945"), "Chainlink ETH/USD Feed");

    // Pyth Network
    map.insert(("ethereum", "0x4305fb66699c3b2702d4d05cf36551390a4c69c6"), "Pyth Network Oracle");
    map.insert(("arbitrum", "0xff1a0f4744e8582df1ae09d5611b887b6a12925c"), "Pyth Network Oracle");

    map.get(&(chain_id, addr.as_str())).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layerzero_v2_detected_on_ethereum() {
        let label = known_infra_label(
            "ethereum",
            "0x1a44076050125825900e736c501f859c50fe728c",
        );
        assert_eq!(label.as_deref(), Some("LayerZero V2 EndpointV2"));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        // Addresses from wallets often come in mixed case (EIP-55 checksum).
        // Our lookup must normalise before comparing.
        let label = known_infra_label(
            "ethereum",
            "0x1A44076050125825900E736C501f859c50fE728c", // mixed case
        );
        assert_eq!(label.as_deref(), Some("LayerZero V2 EndpointV2"));
    }

    #[test]
    fn unknown_address_returns_none() {
        let label = known_infra_label("ethereum", "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef");
        assert!(label.is_none());
    }

    #[test]
    fn wrong_chain_returns_none() {
        // LayerZero V2 address exists on ethereum but not on "solana"
        let label = known_infra_label("solana", "0x1a44076050125825900e736c501f859c50fe728c");
        assert!(label.is_none());
    }

    #[test]
    fn chainlink_feed_detected_on_arbitrum() {
        let label = known_infra_label(
            "arbitrum",
            "0x639fe6ab55c921f74e7fac1ee960c0b6293ba612",
        );
        assert_eq!(label.as_deref(), Some("Chainlink ETH/USD Feed"));
    }

    // TODO: test every entry in the map — each address/chain pair should return the right label
    // TODO: test that no two different chains share an address that returns the same label
    //       (catches copy-paste errors in the address list)
}
