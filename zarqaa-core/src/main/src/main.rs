use zarqaa_adapters::{EvmAdapter, EvmChainConfig};
use zarqaa_types::report::RouteReport;

// `#[tokio::main]` turns our async fn main into a real main by setting up
// the tokio runtime. Without this, you can't use .await anywhere in main.
#[tokio::main]
async fn main() {
    // Load .env if present — allows running locally without exporting vars manually
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

    // Analyze every leg concurrently.
    // futures::future::join_all would be cleaner but to avoid a new dep
    // we just run them sequentially for now — fast enough for a demo.
    let mut legs = Vec::new();
    for addr in &addresses {
        let leg = adapter.analyze_leg(addr).await;
        legs.push(leg);
    }

    let route_verdict = RouteReport::compute_verdict(&legs);

    // ── Print report ─────────────────────────────────────────────────────────
    println!("╔══════════════════════════════════════════════╗");
    println!("║         ZARQA SECURITY REPORT                ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("  TX      : {}", tx_hash);
    println!("  VERDICT : {:?}", route_verdict);
    println!("  LEGS    : {}", legs.len());
    println!();

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
        println!();
    }
}
