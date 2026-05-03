use std::collections::HashMap;
use zarqaa_adapters::{EvmAdapter, EvmChainConfig};
use zarqaa_types::report::{BridgeInfo, DataSource, RouteReport};

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
    let verdict_icon = match route_verdict {
        zarqaa_types::report::Verdict::Green      => "🟢",
        zarqaa_types::report::Verdict::Amber      => "🟡",
        zarqaa_types::report::Verdict::Red        => "🔴",
        zarqaa_types::report::Verdict::Unverified => "⬜",
    };
    println!("╔══════════════════════════════════════════════╗");
    println!("║         ZARQAA SECURITY REPORT               ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("  TX      : {}", tx_hash);
    println!("  VERDICT : {} {:?}", verdict_icon, route_verdict);
    println!("  LEGS    : {}", legs.len());
    println!();

    // ── Per-leg breakdown ─────────────────────────────────────────────────────
    // Collect unique bridge protocols while printing legs so we can print cards once.
    let mut seen_protocols: Vec<String> = Vec::new();
    let mut bridge_cards: HashMap<String, BridgeInfo> = HashMap::new();

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
        if let Some(b) = &leg.bridge_info {
            println!("       ↳ Bridge: {} — see security card below", b.protocol);
            if bridge_cards.insert(b.protocol.clone(), b.clone()).is_none() {
                seen_protocols.push(b.protocol.clone());
            }
        }
        println!();
    }

    // ── Bridge Security Cards (one per unique protocol) ───────────────────────
    if !seen_protocols.is_empty() {
        println!("╔══════════════════════════════════════════════╗");
        println!("║         BRIDGE SECURITY CARDS                ║");
        println!("╚══════════════════════════════════════════════╝");
        println!();

        for protocol in &seen_protocols {
            let b = &bridge_cards[protocol];
            print_bridge_card(b);
            println!();
        }
    }
}

fn expand_controller(controller_type: &str, dvn_count: Option<u32>) -> String {
    match controller_type {
        "don" => {
            let n = dvn_count.map(|n| n.to_string()).unwrap_or_else(|| "multiple".into());
            format!("Chainlink DON ({n} independent node operators)")
        }
        "guardian_council" => {
            let n = dvn_count.map(|n| n.to_string()).unwrap_or_else(|| "19".into());
            format!("Guardian Council ({n} validators, 13-of-{n} threshold)")
        }
        "multisig"  => "Multisig (shared key between core team members)".to_string(),
        "dao"       => "DAO (token holder governance vote)".to_string(),
        "eoa"       => "⚠️  Single wallet (fully centralized — one person controls this)".to_string(),
        other       => other.to_string(),
    }
}

fn data_freshness_label(source: &DataSource) -> String {
    match source {
        DataSource::Static => "reviewed static data".to_string(),
        DataSource::Live { fetched_at } => format!("live — fetched {}", ts_to_date(*fetched_at)),
    }
}

fn print_bridge_card(b: &BridgeInfo) {
    let static_label = data_freshness_label(&b.static_data_source);
    let flags_label  = data_freshness_label(&b.recent_flags_source);

    println!("    ┌─────────────────────────────────────────────────────┐");
    println!("    │  {:<34}  VERDICT: {}  │", b.protocol, b.verdict_label);
    println!("    │  \"{}\"", b.summary);
    println!("    │  Data: {}                   │", static_label);
    println!("    ├─────────────────────────────────────────────────────┤");

    if !b.recent_flags.is_empty() {
        println!("    │  ⚠️  CRITICAL ALERTS ({})", flags_label);
        for flag in &b.recent_flags {
            println!("    │     {flag}");
        }
    } else {
        println!("    │  ✅ NO CRITICAL ALERTS in last 30 days ({})", flags_label);
    }

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  WHO CONTROLS THE BRIDGE?");
    let controller_str = expand_controller(&b.controller_type, b.dvn_count);
    println!("    │  ├─ Operators  : {}", controller_str);
    let quorum = match b.controller_type.as_str() {
        "don"              => format!("All {} must sign off — no single point of failure", b.dvn_count.unwrap_or(5)),
        "guardian_council" => format!("13-of-{} must agree to move funds", b.dvn_count.unwrap_or(19)),
        "multisig"         => "Multiple keyholders must sign — check M-of-N threshold".to_string(),
        _                  => String::new(),
    };
    if !quorum.is_empty() {
        println!("    │  │   ↳ {}", quorum);
    }
    println!("    │  ├─ Controller : {}", b.controller_type);
    let timelock_str = b.upgrade_timelock_days
        .map(|d| format!("{d}-day timelock"))
        .unwrap_or_else(|| "unknown".into());
    println!("    │  └─ Upgrades   : {}", timelock_str);
    if let Some(d) = b.upgrade_timelock_days {
        println!("    │       ↳ You have {d} days to notice any change and exit before it takes effect");
    }

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  SECURITY HISTORY");
    match b.past_exploits_usd {
        None | Some(0) => println!("    │  ├─ Past Exploits : None on record"),
        Some(usd)      => println!("    │  ├─ Past Exploits : ${} — review before large transfers", format_usd(usd)),
    }
    if let Some(note) = &b.past_exploit_note {
        println!("    │  │   ↳ {note}");
    }
    println!("    │  ├─ Last Audit    : {}", b.last_audit.as_deref().unwrap_or("Unknown"));
    println!("    │  └─ Bug Bounty    : {}",
        b.bug_bounty_usd
            .map(|n| format!("${} — researchers incentivized to find bugs", format_usd(n)))
            .unwrap_or_else(|| "None".into())
    );

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  ACTIVE PROTECTIONS");
    if b.has_rate_limits {
        println!("    │  ├─ Rate Limits    : ✅ Yes");
        println!("    │  │   ↳ Caps daily outflow — slows down large theft attempts");
    } else {
        println!("    │  ├─ Rate Limits    : ❌ No — unlimited outflow possible");
    }
    if b.has_circuit_breaker {
        println!("    │  ├─ Circuit Breaker: ✅ Yes");
        println!("    │  │   ↳ Can pause the bridge automatically on anomalous activity");
    } else {
        println!("    │  ├─ Circuit Breaker: ❌ No — no automatic stop");
    }
    println!("    │  └─ Emergency Pause: {}", b.emergency_pause_by.as_deref().unwrap_or("Unknown"));

    println!("    ├─────────────────────────────────────────────────────┤");
    let flags_line = if b.recent_flags.is_empty() {
        format!("🟢 None ({})", flags_label)
    } else {
        b.recent_flags.join(", ")
    };
    println!("    │  RECENT FLAGS — Last 30 Days: {flags_line}");

    println!("    ├─────────────────────────────────────────────────────┤");
    println!("    │  🎯 ZARQAA VERDICT: {}", b.verdict_label.to_uppercase());
    println!("    │  {}", b.verdict_summary);
    if let Some(dest) = &b.destination_chain {
        println!("    │  🌐 Cross-chain delivery to: {dest}");
        println!("    │     ↳ Destination-chain analysis coming soon");
    }
    println!("    └─────────────────────────────────────────────────────┘");
}

fn ts_to_date(ts: u64) -> String {
    let mut days = ts / 86400;
    let mut y = 1970u64;
    loop {
        let dim = if is_leap(y) { 366 } else { 365 };
        if days < dim { break; }
        days -= dim;
        y += 1;
    }
    let months = if is_leap(y) {
        [31u64, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for &dim in &months {
        if days < dim { break; }
        days -= dim;
        m += 1;
    }
    format!("{y}-{m:02}-{d:02}", d = days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
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
