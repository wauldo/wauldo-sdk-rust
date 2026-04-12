//! Quickstart — explore the full Wauldo SDK without a running server.
//!
//! Run with: `cargo run --example quickstart`

use wauldo::{ChatMessage, ChatRequest, FactCheckRequest, MockHttpClient, VerifyCitationRequest};

#[tokio::main]
async fn main() {
    // MockHttpClient::with_defaults() ships realistic sample data for every
    // endpoint, so you can explore the full API surface immediately.
    let client = MockHttpClient::with_defaults();

    println!("=== Wauldo SDK Quickstart ===\n");

    // ── 1. Upload a document ────────────────────────────────────────────
    let upload = client
        .rag_upload(
            "Our refund policy allows returns within 60 days of purchase. \
             Customers must provide a valid receipt. \
             Refunds are processed within 5 business days after approval.",
            Some("refund_policy.txt".into()),
        )
        .await
        .expect("upload failed");

    println!("1. Document uploaded");
    println!("   Document ID : {}", upload.document_id);
    println!("   Chunks      : {}", upload.chunks_count);
    println!();

    // ── 2. Query the document ───────────────────────────────────────────
    let result = client
        .rag_query("What is the refund policy?", None)
        .await
        .expect("query failed");

    println!("2. RAG Query");
    println!("   Answer     : {}", result.answer);
    if let Some(audit) = &result.audit {
        println!("   Confidence : {:.0}%", audit.confidence * 100.0);
        println!("   Grounded   : {}", audit.grounded);
        println!("   Model      : {}", audit.model);
        println!("   Latency    : {} ms", audit.latency_ms);
    }
    println!("   Sources:");
    for (i, src) in result.sources.iter().enumerate() {
        println!("     [{}] score={:.2} | {}", i + 1, src.score, src.content);
    }
    println!();

    // ── 3. Fact-check a claim against the source ────────────────────────
    let fc = client
        .fact_check(FactCheckRequest::new(
            "Returns are accepted within 60 days.",
            "Our refund policy allows returns within 60 days of purchase.",
        ))
        .await
        .expect("fact-check failed");

    println!("3. Fact-Check");
    println!("   Verdict           : {}", fc.verdict);
    println!("   Action            : {}", fc.action);
    println!(
        "   Hallucination rate: {:.0}%",
        fc.hallucination_rate * 100.0
    );
    println!(
        "   Claims verified   : {}/{}",
        fc.supported_claims, fc.total_claims
    );
    for claim in &fc.claims {
        let status = if claim.supported { "OK" } else { "FAIL" };
        println!(
            "     [{}] \"{}\" (confidence: {:.0}%)",
            status,
            claim.text,
            claim.confidence * 100.0,
        );
    }
    println!();

    // ── 4. Verify citation coverage ─────────────────────────────────────
    let vc = client
        .verify_citation(VerifyCitationRequest::new(
            "Returns are within 60 days [1]. A receipt is needed [2]. \
             Refunds are processed quickly.",
        ))
        .await
        .expect("verify-citation failed");

    println!("4. Citation Verification");
    println!("   Citation ratio : {:.0}%", vc.citation_ratio * 100.0);
    println!("   Sufficient     : {}", vc.has_sufficient_citations);
    println!("   Sentences      : {}", vc.sentence_count);
    println!("   Citations found: {}", vc.citation_count);
    if let Some(phantoms) = vc.phantom_count {
        println!("   Phantom refs   : {}", phantoms);
    }
    if !vc.uncited_sentences.is_empty() {
        println!("   Uncited:");
        for s in &vc.uncited_sentences {
            println!("     - \"{}\"", s);
        }
    }
    println!();

    // ── 5. Analytics & Insights ─────────────────────────────────────────
    let insights = client.get_insights().await.expect("insights failed");

    println!("5. Insights (ROI)");
    println!("   Total requests        : {}", insights.total_requests);
    println!(
        "   Intelligence requests : {}",
        insights.intelligence_requests
    );
    println!(
        "   Tokens saved          : {} ({:.0}%)",
        insights.tokens.saved_total, insights.tokens.saved_percent_avg
    );
    println!(
        "   Estimated cost saved  : ${:.2}",
        insights.cost.estimated_usd_saved
    );
    println!();

    // ── 6. Chat completion ──────────────────────────────────────────────
    let chat = client
        .chat(ChatRequest::new(
            "qwen2.5:7b",
            vec![ChatMessage::user(
                "Explain ownership in Rust in one sentence.",
            )],
        ))
        .await
        .expect("chat failed");

    println!("6. Chat Completion");
    println!("   Model  : {}", chat.model);
    println!("   Answer : {}", chat.content());
    println!("   Tokens : {}", chat.usage.total_tokens);
    println!();

    // ── 7. List models ──────────────────────────────────────────────────
    let models = client.list_models().await.expect("list_models failed");

    println!("7. Available Models");
    for m in &models.data {
        println!("   - {} (by {})", m.id, m.owned_by);
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
