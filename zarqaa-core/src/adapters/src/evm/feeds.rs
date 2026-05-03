use reqwest::Client;
use serde::Deserialize;
use zarqaa_types::report::DataSource;

// DefiLlama hacks tracker response shape.
// Dates may be a unix timestamp (number) or ISO string depending on the entry.
#[derive(Deserialize)]
struct HackEntry {
    date: Option<serde_json::Value>,
    name: Option<String>,
    #[serde(rename = "defillamaId")]
    defillama_id: Option<String>,
    amount: Option<f64>,
    technique: Option<String>,
    chain: Option<String>,
}

// Fetch recent incidents mentioning this protocol from the DefiLlama hacks tracker.
// Returns (flags, data_source). Never returns Err — any failure yields empty flags
// with DataSource::Static so the rest of the analysis continues unaffected.
pub async fn fetch_recent_flags(protocol: &str, http: &Client) -> (Vec<String>, DataSource) {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let cutoff_ts = now_ts.saturating_sub(30 * 24 * 3600);

    let resp = match http
        .get("https://hacksapi.defillama.com/")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return (vec![], DataSource::Static),
    };

    let hacks: Vec<HackEntry> = match resp.json().await {
        Ok(h) => h,
        Err(_) => return (vec![], DataSource::Static),
    };

    let proto_lower = protocol.to_lowercase();
    // Match on protocol name or the defillamaId slug
    let flags: Vec<String> = hacks
        .iter()
        .filter(|h| {
            let name_match = h.name.as_deref()
                .map(|n| n.to_lowercase().contains(&proto_lower))
                .unwrap_or(false);
            let id_match = h.defillama_id.as_deref()
                .map(|id| id.to_lowercase().contains(&proto_lower))
                .unwrap_or(false);
            name_match || id_match
        })
        .filter(|h| entry_ts(h).map(|ts| ts >= cutoff_ts).unwrap_or(false))
        .filter_map(|h| {
            let name = h.name.as_deref()?;
            let amount = h.amount.map(|a| {
                if a >= 1_000_000.0 {
                    format!("${:.0}M", a / 1_000_000.0)
                } else if a >= 1_000.0 {
                    format!("${:.0}K", a / 1_000.0)
                } else {
                    format!("${:.0}", a)
                }
            });
            let chain = h.chain.as_deref().unwrap_or("");
            let technique = h.technique.as_deref().unwrap_or("");
            let date_str = entry_ts(h)
                .map(ts_to_date)
                .unwrap_or_default();
            let mut parts = vec![date_str, name.to_string()];
            if let Some(a) = amount { parts.push(a); }
            if !chain.is_empty() { parts.push(format!("on {chain}")); }
            if !technique.is_empty() { parts.push(format!("via {technique}")); }
            Some(parts.join(" "))
        })
        .collect();

    let source = DataSource::Live { fetched_at: now_ts };
    (flags, source)
}

fn entry_ts(h: &HackEntry) -> Option<u64> {
    match &h.date {
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        Some(serde_json::Value::String(s)) => s.parse::<u64>().ok(),
        _ => None,
    }
}

// Convert unix timestamp to a YYYY-MM-DD string without external crates.
fn ts_to_date(ts: u64) -> String {
    // Days since epoch
    let mut days = ts / 86400;
    let mut y = 1970u64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
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
