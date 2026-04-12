//! Analytics demo — explore insights, analytics, and traffic endpoints.
//!
//! Run with: `cargo run --example analytics_demo`

use wauldo::MockHttpClient;

#[tokio::main]
async fn main() {
    let client = MockHttpClient::with_defaults();

    println!("=== Wauldo Analytics Demo ===\n");

    // ── 1. ROI Insights ─────────────────────────────────────────────────
    let insights = client.get_insights().await.expect("insights failed");

    println!("--- ROI Insights (/v1/insights) ---");
    println!("  TIG Key              : {}", insights.tig_key);
    println!("  Total requests       : {}", insights.total_requests);
    println!(
        "  Intelligence requests: {}",
        insights.intelligence_requests
    );
    println!("  Fallback requests    : {}", insights.fallback_requests);
    println!();
    println!("  Tokens:");
    println!(
        "    Baseline total     : {}",
        insights.tokens.baseline_total
    );
    println!("    Real total         : {}", insights.tokens.real_total);
    println!("    Saved              : {}", insights.tokens.saved_total);
    println!(
        "    Avg savings        : {:.1}%",
        insights.tokens.saved_percent_avg
    );
    println!();
    println!(
        "  Estimated USD saved  : ${:.2}",
        insights.cost.estimated_usd_saved
    );
    println!();

    // ── 2. Usage Analytics ──────────────────────────────────────────────
    let analytics = client
        .get_analytics(Some(60))
        .await
        .expect("analytics failed");

    println!("--- Usage Analytics (/v1/analytics?minutes=60) ---");
    println!("  Cache:");
    println!(
        "    Total requests     : {}",
        analytics.cache.total_requests
    );
    println!(
        "    Hit rate           : {:.1}%",
        analytics.cache.cache_hit_rate * 100.0
    );
    println!(
        "    Avg latency        : {:.0} ms",
        analytics.cache.avg_latency_ms
    );
    println!(
        "    P95 latency        : {:.0} ms",
        analytics.cache.p95_latency_ms
    );
    println!();
    println!("  Tokens:");
    println!(
        "    Baseline           : {}",
        analytics.tokens.total_baseline
    );
    println!("    Real               : {}", analytics.tokens.total_real);
    println!("    Saved              : {}", analytics.tokens.total_saved);
    println!(
        "    Avg savings        : {:.1}%",
        analytics.tokens.avg_savings_percent
    );
    println!();
    let hours = analytics.uptime_secs / 3600;
    let mins = (analytics.uptime_secs % 3600) / 60;
    println!("  Uptime               : {}h {}m", hours, mins);
    println!();

    // ── 3. Traffic Summary ──────────────────────────────────────────────
    let traffic = client
        .get_analytics_traffic()
        .await
        .expect("traffic failed");

    println!("--- Traffic Summary (/v1/analytics/traffic) ---");
    println!("  Requests today       : {}", traffic.total_requests_today);
    println!("  Tokens today         : {}", traffic.total_tokens_today);
    println!(
        "  Error rate           : {:.1}%",
        traffic.error_rate * 100.0
    );
    println!("  Avg latency          : {} ms", traffic.avg_latency_ms);
    println!("  P95 latency          : {} ms", traffic.p95_latency_ms);
    println!();
    println!("  Top tenants:");
    for t in &traffic.top_tenants {
        println!(
            "    {} | {} reqs | {} tokens | {:.0}% success | {} ms avg",
            t.tenant_id,
            t.requests_today,
            t.tokens_used,
            t.success_rate * 100.0,
            t.avg_latency_ms,
        );
    }
    println!();

    println!("=== Done! ===");
    println!();
    println!("To use the real API, replace MockHttpClient with HttpClient:");
    println!();
    println!("  use wauldo::{{HttpClient, HttpConfig}};");
    println!("  let client = HttpClient::new(");
    println!("      HttpConfig::new(\"https://api.wauldo.com\").with_api_key(\"YOUR_KEY\"),");
    println!("  )?;");
}
