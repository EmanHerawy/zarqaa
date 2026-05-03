use zarqaa_types::report::{LegReport, MevRisk, MevRiskLevel};

const FLASHBOTS_URL: &str = "https://protect.flashbots.net";
const MEV_BLOCKER_URL: &str = "https://mevblocker.io";

pub fn assess(legs: &[LegReport], value_wei: Option<u64>) -> MevRisk {
    let dex_signals: Vec<String> = legs
        .iter()
        .filter_map(|l| l.infra_kind.as_deref())
        .filter(|label| is_dex(label))
        .map(str::to_string)
        .collect();

    let has_bridge = legs.iter().any(|l| l.bridge_info.is_some());
    let dex_count = dex_signals.len();

    if dex_count >= 2 {
        return MevRisk {
            risk_level: MevRiskLevel::High,
            summary: format!(
                "Multiple DEX interactions ({}) — arbitrage opportunity is visible to \
                 MEV searchers in the public mempool",
                dex_signals.join(", ")
            ),
            recommendation: format!(
                "Use a private RPC to hide this transaction from searchers before inclusion. \
                 Options: Flashbots Protect ({FLASHBOTS_URL}) or MEV Blocker ({MEV_BLOCKER_URL})."
            ),
            signals: dex_signals,
        };
    }

    if dex_count == 1 {
        return MevRisk {
            risk_level: MevRiskLevel::High,
            summary: format!(
                "DEX swap detected ({}) — sandwich attack risk if sent to public mempool",
                dex_signals[0]
            ),
            recommendation: format!(
                "Use a private RPC so searchers cannot see this transaction before it lands. \
                 Flashbots Protect: {FLASHBOTS_URL} — MEV Blocker: {MEV_BLOCKER_URL}"
            ),
            signals: dex_signals,
        };
    }

    // No DEX — check bridge + large value
    if has_bridge {
        if let Some(wei) = value_wei {
            if wei > 1_000_000_000_000_000_000 {
                // > 1 ETH through a bridge
                let eth = wei / 1_000_000_000_000_000_000;
                return MevRisk {
                    risk_level: MevRiskLevel::Medium,
                    summary: format!(
                        "Large value ({eth} ETH) bridged via public mempool — \
                         may attract front-running"
                    ),
                    recommendation: "Consider a private RPC for large transfers.".to_string(),
                    signals: vec![format!("{eth} ETH value")],
                };
            }
        }
        return MevRisk {
            risk_level: MevRiskLevel::Low,
            summary: "Bridge transaction — low MEV surface area".to_string(),
            recommendation:
                "Bridge transactions are generally safe on the public mempool. \
                 No DEX interactions detected."
                    .to_string(),
            signals: vec![],
        };
    }

    MevRisk {
        risk_level: MevRiskLevel::None,
        summary: "No DEX or high-value interactions detected — minimal MEV risk".to_string(),
        recommendation: "Standard RPC is fine for this transaction.".to_string(),
        signals: vec![],
    }
}

fn is_dex(label: &str) -> bool {
    let l = label.to_lowercase();
    l.contains("uniswap")
        || l.contains("curve")
        || l.contains("balancer")
        || l.contains("1inch")
        || l.contains("0x exchange")
        || l.contains("swap router")
        || l.contains("universal router")
}
