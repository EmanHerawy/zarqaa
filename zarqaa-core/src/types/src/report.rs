use serde::{Deserialize, Serialize};

// A contract address on a specific chain.
// We store chain as a string ("ethereum", "arbitrum") not an enum —
// this way adding a new chain never requires changing this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAddress {
    pub chain: String,
    pub address: String,
}

// Where a field's data came from and how fresh it is.
//   Static   — reviewed, hardcoded; accurate but may lag quarterly updates
//   Live     — just fetched from an external source this run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum DataSource {
    Static,
    Live { fetched_at: u64 }, // unix timestamp
}

// The four possible outcomes for a contract leg.
// Ordering matters: Green < Amber < Unverified < Red (weakest leg wins).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Green,
    Amber,
    Red,
    Unverified,
}

impl Verdict {
    // Combine two verdicts — always return the worse one.
    // Used to compute route verdict from all legs.
    pub fn weaken(self, other: Verdict) -> Verdict {
        match (self, other) {
            (Verdict::Red, _) | (_, Verdict::Red)               => Verdict::Red,
            (Verdict::Unverified, _) | (_, Verdict::Unverified) => Verdict::Unverified,
            (Verdict::Amber, _) | (_, Verdict::Amber)           => Verdict::Amber,
            _                                                    => Verdict::Green,
        }
    }
}

// Bridge Security Card — all data needed to answer the 5 user questions:
//   1. Who controls the bridge?
//   2. Has it been hacked before?
//   3. Does it stop itself if something goes wrong?
//   4. Any recent warnings?
//   5. Should I proceed?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeInfo {
    pub protocol: String,
    pub summary: String,                 // one-line plain English trust model

    // WHO CONTROLS — static, changes quarterly at most
    pub dvn_count: Option<u32>,          // number of verifiers/validators
    pub controller_type: String,         // "multisig" | "dao" | "eoa" | "don" | "guardian_council"
    pub upgrade_timelock_days: Option<u32>,
    pub relayer_type: String,            // "contract" | "eoa" | "don"

    // SECURITY HISTORY — static (historical facts don't change)
    pub past_exploits_usd: Option<u64>,  // 0 = none, Some(n) = exploited for $n
    pub past_exploit_note: Option<String>,
    pub last_audit: Option<String>,      // e.g. "Jan 2026 (Hexens)"
    pub bug_bounty_usd: Option<u64>,

    // ACTIVE PROTECTIONS — static (structural properties of the protocol)
    pub has_rate_limits: bool,
    pub has_circuit_breaker: bool,
    pub emergency_pause_by: Option<String>,

    // RECENT FLAGS (last 30 days) — fetched live from DefiLlama/Rekt feeds
    pub recent_flags: Vec<String>,
    pub recent_flags_source: DataSource,

    // VERDICT
    pub verdict_label: String,           // "Proceed" | "Review" | "Stop"
    pub verdict_summary: String,         // plain English why

    // CROSS-CHAIN
    pub centralization_risk: String,     // "low" | "medium" | "high"
    pub destination_chain: Option<String>,

    // Whether the static fields come from reviewed hardcoded data or on-chain reads.
    // Phase 2 replaces static fields with live on-chain reads.
    pub static_data_source: DataSource,
}

// Everything we know about one contract in the transaction path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegReport {
    pub address: String,
    pub chain: String,
    pub verdict: Verdict,
    pub source_verified: bool,
    pub is_proxy: bool,
    pub proxy_implementation: Option<String>,
    pub infra_kind: Option<String>,      // e.g. "Chainlink CCIP Router"
    pub bridge_info: Option<BridgeInfo>, // present when infra_kind is a bridge/oracle
    pub notes: Vec<String>,
}

// MEV exposure assessment for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MevRiskLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevRisk {
    pub risk_level: MevRiskLevel,
    pub summary: String,       // one sentence plain English
    pub recommendation: String,
    pub signals: Vec<String>,  // what triggered this (e.g. "Uniswap V3 Router detected")
}

// Populated on the intent path — shows how the input was interpreted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResolution {
    pub normalized_to: String,
    pub decoded_call: Option<String>,   // e.g. "exactInputSingle(...)"
    pub unresolved_legs: Vec<String>,   // dynamic addresses that could not be resolved
}

// The complete report returned to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub tx_hash: Option<String>,        // None on intent path
    pub chain: String,
    pub route_verdict: Verdict,
    pub legs: Vec<LegReport>,
    pub mev_risk: Option<MevRisk>,
    pub intent_resolution: Option<IntentResolution>,
}

impl RouteReport {
    // Fold all leg verdicts into one using weaken().
    // A single Red leg makes the whole route Red.
    pub fn compute_verdict(legs: &[LegReport]) -> Verdict {
        legs.iter()
            .map(|l| l.verdict.clone())
            .fold(Verdict::Green, |acc, v| acc.weaken(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a minimal LegReport with just a verdict set.
    fn leg(verdict: Verdict) -> LegReport {
        LegReport {
            address: "0xdead".to_string(),
            chain: "ethereum".to_string(),
            verdict,
            source_verified: false,
            is_proxy: false,
            proxy_implementation: None,
            infra_kind: None,
            bridge_info: None,
            notes: vec![],
        }
    }

    // --- Verdict::weaken ---

    #[test]
    fn green_green_is_green() {
        assert_eq!(Verdict::Green.weaken(Verdict::Green), Verdict::Green);
    }

    #[test]
    fn green_amber_is_amber() {
        assert_eq!(Verdict::Green.weaken(Verdict::Amber), Verdict::Amber);
    }

    #[test]
    fn amber_green_is_amber() {
        // weaken is symmetric — order shouldn't matter
        assert_eq!(Verdict::Amber.weaken(Verdict::Green), Verdict::Amber);
    }

    #[test]
    fn red_beats_everything() {
        assert_eq!(Verdict::Green.weaken(Verdict::Red),      Verdict::Red);
        assert_eq!(Verdict::Amber.weaken(Verdict::Red),      Verdict::Red);
        assert_eq!(Verdict::Unverified.weaken(Verdict::Red), Verdict::Red);
        assert_eq!(Verdict::Red.weaken(Verdict::Green),      Verdict::Red);
    }

    #[test]
    fn unverified_beats_amber_and_green() {
        assert_eq!(Verdict::Green.weaken(Verdict::Unverified), Verdict::Unverified);
        assert_eq!(Verdict::Amber.weaken(Verdict::Unverified), Verdict::Unverified);
    }

    // --- RouteReport::compute_verdict ---

    #[test]
    fn empty_legs_is_green() {
        // No legs = no risk found = Green (vacuously safe)
        assert_eq!(RouteReport::compute_verdict(&[]), Verdict::Green);
    }

    #[test]
    fn all_green_legs_is_green() {
        let legs = vec![leg(Verdict::Green), leg(Verdict::Green)];
        assert_eq!(RouteReport::compute_verdict(&legs), Verdict::Green);
    }

    #[test]
    fn one_red_leg_makes_route_red() {
        // The "weakest leg wins" rule — one bad contract poisons the route
        let legs = vec![
            leg(Verdict::Green),
            leg(Verdict::Red),
            leg(Verdict::Green),
        ];
        assert_eq!(RouteReport::compute_verdict(&legs), Verdict::Red);
    }

    #[test]
    fn one_unverified_among_green_is_unverified() {
        let legs = vec![leg(Verdict::Green), leg(Verdict::Unverified)];
        assert_eq!(RouteReport::compute_verdict(&legs), Verdict::Unverified);
    }

    // TODO: test that serde round-trips Verdict correctly (Green serialises as "green")
    // TODO: test ChainAddress serialisation format
}
