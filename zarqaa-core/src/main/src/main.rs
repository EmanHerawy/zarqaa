use std::collections::HashMap;
use zarqaa_adapters::{EvmAdapter, EvmChainConfig};
use zarqaa_types::report::{BridgeInfo, RouteReport};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let tx_hash = std::env::args().nth(1)
        .expect("Usage: zarqaa <tx_hash>");

    let config = EvmChainConfig::ethereum_from_env();
    let adapter = EvmAdapter::new(config);

    println!("Resolving legs for {} ...\n", tx_hash);

    let addresses = match adapter.resolve_legs(&tx_hash).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error resolving tx: {e}");
            std::process::exit(1);
        }
    };

    println!("Found {} leg(s). Analyzing ...\n", addresses.len());

    let mut legs = Vec::new();
    for addr in &addresses {
        let leg = adapter.analyze_leg(addr).await;
        legs.push(leg);
    }

    let route_verdict = RouteReport::compute_verdict(&legs);

    // ── Header ────────────────────────────────────────────────────────────────
    println!("╔══════════════════════════════════════════════╗");
    println!("║         ZARQAA SECURITY REPORT               ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("  TX      : {}", tx_hash);
    println!("  VERDICT : {:?}", route_verdict);
    println!("  LEGS    : {}", legs.len());
    println!();

    // ── Legs ──────────────────────────────────────────────────────────────────
    for (i, leg) in legs.iter().enumerate() {
        let icon = match leg.verdict {
            zarqaa_types::report::Verdict::Green      => "🟢",
            zarqaa_types::report::Verdict::Amber      => "🟡",
            zarqaa_types::report::Verdict::Red        => "🔴",
            zarqaa_types::report::Verdict::Unverified => "⬜",
        };
        println!("  {} Leg {} — {}", icon, i + 1, leg.address);
        for note in &leg.notes {
            println!("       • {}", note);
        }
        // If this leg belongs to a bridge, just reference the protocol name.
        // The full security card is printed once at the end of the report.
        if let Some(b) = &leg.bridge_info {
            println!("       ↳ Bridge: {} — see security card below", b.protocol);
        }
        println!();
    }

    // ── Bridge Security Cards (one per unique protocol) ───────────────────────
    // Collect unique bridge infos keyed by protocol name, preserving first occurrence.
    let mut seen_protocols: Vec<String> = Vec::new();
    let mut bridge_cards: HashMap<String, &BridgeInfo> = HashMap::new();
    for leg in &legs {
        if let Some(b) = &leg.bridge_info {
            if bridge_cards.insert(b.protocol.clone(), b).is_none() {
                seen_protocols.push(b.protocol.clone());
            }
        }
    }

    if !seen_protocols.is_empty() {
        println!("╔══════════════════════════════════════════════╗");
        println!("║         BRIDGE SECURITY CARDS                ║");
        println!("╚══════════════════════════════════════════════╝");
        println!();

        for protocol in &seen_protocols {
            let b = bridge_cards[protocol];
            print_bridge_card(b);
            println!();
        }
    }
}

fn print_bridge_card(b: &BridgeInfo) {
    let mock_tag = if b.is_mocked { " [MOCK]" } else { "" };

    println!("    ┌─────────────────────────────────────────────────────┐");
    println!("    │  {:<34}  VERDICT: {}  │", b.protocol, b.verdict_label);
    println!("    │  \"{}\"", b.summary);
    println!("    ├─────────────────────────────────────────────────────┤");

    if !b.recent_flags.is_empty() {
        println!("    │  ⚠️  CRITICAL ALERTS");
        for flag in &b.recent_flags {
            println!("    │     {flag}");
        }
    } else {
        println!("    │  ✅ NO CRITICAL ALERTS");
    }

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  WHO CONTROLS THE BRIDGE?");
    println!("    │  ├─ Verifiers  : {}{}",
        b.dvn_count.map(|n| n.to_string()).unwrap_or_else(|| "unknown".into()),
        match b.controller_type.as_str() {
            "don"              => " DON nodes",
            "guardian_council" => " Guardians (13-of-19 threshold)",
            _                  => " nodes",
        }
    );
    println!("    │  ├─ Controller : {}", b.controller_type);
    println!("    │  └─ Upgrades   : {}",
        b.upgrade_timelock_days
            .map(|d| format!("{d}-day timelock"))
            .unwrap_or_else(|| "unknown".into())
    );

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  SECURITY HISTORY{mock_tag}");
    match b.past_exploits_usd {
        None | Some(0) => println!("    │  ├─ Past Exploits : None"),
        Some(usd)      => println!("    │  ├─ Past Exploits : ${}", format_usd(usd)),
    }
    if let Some(note) = &b.past_exploit_note {
        println!("    │  │   ↳ {note}");
    }
    println!("    │  ├─ Last Audit    : {}", b.last_audit.as_deref().unwrap_or("Unknown"));
    println!("    │  └─ Bug Bounty    : {}",
        b.bug_bounty_usd
            .map(|n| format!("${}", format_usd(n)))
            .unwrap_or_else(|| "Unknown".into())
    );

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  ACTIVE PROTECTIONS{mock_tag}");
    println!("    │  ├─ Rate Limits    : {}", if b.has_rate_limits { "✅ Yes" } else { "❌ No" });
    println!("    │  ├─ Circuit Breaker: {}", if b.has_circuit_breaker { "✅ Yes" } else { "❌ No" });
    println!("    │  └─ Emergency Pause: {}", b.emergency_pause_by.as_deref().unwrap_or("Unknown"));

    println!("    ├─────────────────────────────────────────────────────┤");
    let flags_line = if b.recent_flags.is_empty() {
        "🟢 None".to_string()
    } else {
        b.recent_flags.join(", ")
    };
    println!("    │  RECENT FLAGS (Last 30 Days){mock_tag}: {flags_line}");

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  🎯 ZARQAA VERDICT: {}", b.verdict_label.to_uppercase());
    println!("    │  {}", b.verdict_summary);
    if let Some(dest) = &b.destination_chain {
        println!("    │  Destination: {dest} (tracking: CROSS_CHAIN_UNSUPPORTED)");
    }
    println!("    └─────────────────────────────────────────────────────┘");
}

fn format_usd(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.0}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}
