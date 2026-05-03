use serde::{Deserialize, Serialize};

// A contract address on a specific chain.
// We store chain as a string ("ethereum", "arbitrum") not an enum —
// this way adding a new chain never requires changing this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAddress {
    pub chain: String,
    pub address: String,
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

// Everything we know about one contract in the transaction path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegReport {
    pub address: String,
    pub chain: String,
    pub verdict: Verdict,
    pub source_verified: bool,
    pub is_proxy: bool,
    pub proxy_implementation: Option<String>,
    pub infra_kind: Option<String>,  // e.g. "LayerZero V2 EndpointV2"
    pub notes: Vec<String>,          // human-readable findings for this leg
}

// The complete report returned to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteReport {
    pub tx_hash: String,
    pub chain: String,
    pub route_verdict: Verdict,
    pub legs: Vec<LegReport>,
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
