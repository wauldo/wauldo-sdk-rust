<h1 align="center">Wauldo Rust SDK</h1>

<p align="center">
  <strong>Verified AI answers from your documents — or no answer at all.</strong>
</p>

<p align="center">
  Most RAG APIs guess. Wauldo verifies.
</p>

<p align="center">
  <b>0% hallucination</b> &nbsp;|&nbsp; 83% accuracy &nbsp;|&nbsp; 61 eval tasks &nbsp;|&nbsp; 14 LLMs tested
</p>

<p align="center">
  <a href="https://crates.io/crates/wauldo"><img src="https://img.shields.io/crates/v/wauldo.svg" alt="crates.io" /></a>&nbsp;
  <a href="https://crates.io/crates/wauldo"><img src="https://img.shields.io/crates/d/wauldo.svg" alt="Downloads" /></a>&nbsp;
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust" />&nbsp;
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT" />
</p>

<p align="center">
  <a href="https://wauldo.com/demo">Demo</a> &bull;
  <a href="https://wauldo.com/docs">Docs</a> &bull;
  <a href="https://rapidapi.com/binnewzzin/api/smart-rag-api">Free API Key</a> &bull;
  <a href="https://dev.to/wauldo/how-we-achieved-0-hallucination-rate-in-our-rag-api-with-benchmarks-4g54">Benchmarks</a>
</p>

---

## Quickstart (30 seconds)

```toml
[dependencies]
wauldo = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use wauldo::{HttpClient, HttpConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::new(
        HttpConfig::new("https://api.wauldo.com").with_api_key("YOUR_API_KEY"),
    )?;

    // Upload a document
    client.rag_upload("Our refund policy allows returns within 60 days...", Some("policy.txt".into())).await?;

    // Ask a question — answer is verified against the source
    let result = client.rag_query("What is the refund policy?", None).await?;
    println!("Answer: {}", result.answer);
    println!("Grounded: {}", result.grounded());
    Ok(())
}
```

```
Output:
Answer: Returns are accepted within 60 days of purchase.
Grounded: true | Confidence: 92%
```

[Try the demo](https://wauldo.com/demo) | [Get a free API key](https://rapidapi.com/binnewzzin/api/smart-rag-api)

---

## Why Wauldo (and not standard RAG)

**Typical RAG pipeline**

```
retrieve → generate → hope it's correct
```

**Wauldo pipeline**

```
retrieve → extract facts → generate → verify → return or refuse
```

If the answer can't be verified, it returns **"insufficient evidence"** instead of guessing.

### See the difference

```
Document: "Refunds are processed within 60 days"

Typical RAG:  "Refunds are processed within 30 days"     ← wrong
Wauldo:       "Refunds are processed within 60 days"     ← verified
              or "insufficient evidence" if unclear       ← safe
```

---

## Try locally (no server needed)

Explore every feature using `MockHttpClient` -- no API key, no server, no network:

```rust
use wauldo::MockHttpClient;

#[tokio::main]
async fn main() {
    let client = MockHttpClient::with_defaults();

    // Upload + query
    let _ = client.rag_upload("Your document text...", None).await.unwrap();
    let result = client.rag_query("What is the refund policy?", None).await.unwrap();
    println!("Answer: {}", result.answer);
    println!("Grounded: {}", result.grounded().unwrap_or(false));

    // Fact-check
    let fc = client.fact_check(wauldo::FactCheckRequest::new(
        "Returns within 60 days.", "Policy allows returns within 60 days.",
    )).await.unwrap();
    println!("Verdict: {}", fc.verdict);

    // Analytics
    let insights = client.get_insights().await.unwrap();
    println!("Tokens saved: {}", insights.tokens.saved_total);
}
```

Run the full quickstart example:

```bash
cargo run --example quickstart
cargo run --example analytics_demo
```

---

## Examples

### Upload a PDF and ask questions

```rust
// Upload — text extraction + quality scoring happens server-side
let upload = client.upload_file("contract.pdf", Some("Q3 Contract".into()), None).await?;
println!("Extracted {} chunks, quality: {}", upload.chunks_count, upload.quality_label);

// Query
let result = client.rag_query("What are the payment terms?", None).await?;
println!("Answer: {}", result.answer);
println!("Confidence: {:.0}%", result.confidence() * 100.0);
println!("Grounded: {}", result.grounded());
```

### Fact-check any LLM output

```rust
let result = client.fact_check(
    "Returns are accepted within 60 days.",
    "Our policy allows returns within 14 days.",
    Some("lexical"),
).await?;
println!("Verdict: {}", result.verdict);        // "rejected"
println!("Action: {}", result.action);           // "block"
println!("Reason: {}", result.claims[0].reason); // "numerical_mismatch"
```

### Chat (OpenAI-compatible)

```rust
use wauldo::{ChatRequest, ChatMessage};

let req = ChatRequest::new("auto", vec![ChatMessage::user("Explain ownership in Rust")]);
let resp = client.chat(req).await?;
println!("{}", resp.content());
```

### Streaming

```rust
let req = ChatRequest::new("auto", vec![ChatMessage::user("Hello!")]);
let mut rx = client.chat_stream(req).await?;
while let Some(chunk) = rx.recv().await {
    print!("{}", chunk.unwrap_or_default());
}
```

### Conversation

```rust
let mut conv = client.conversation()
    .with_system("You are an expert on Rust programming.")
    .with_model("auto");
let reply = conv.say("What is the borrow checker?").await?;
let follow_up = conv.say("Give me an example").await?;
```

---

## Features

- **Pre-generation fact extraction** — numbers, dates, limits injected as constraints
- **Post-generation grounding check** — every answer verified against sources
- **Citation validation** — detects phantom references
- **Analytics & Insights** — track token savings, cache performance, cost per hour, and per-tenant traffic
- **Fact-check API** — verify any claim against any source (3 modes)
- **Native PDF/DOCX upload** — server-side extraction with quality scoring
- **Smart model routing** — auto-selects cheapest model that meets quality
- **OpenAI-compatible** — swap your `base_url`, keep your existing code
- **Type-safe** — full Rust type system, no unwrap in production

---

## Error Handling

```rust
use wauldo::Error;

match client.chat(req).await {
    Ok(resp) => println!("{}", resp.content()),
    Err(Error::Server { code, message, .. }) => eprintln!("Server error [{}]: {}", code, message),
    Err(Error::Connection(msg)) => eprintln!("Connection failed: {}", msg),
    Err(Error::Timeout(msg)) => eprintln!("Timeout: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

## RapidAPI

```rust
let config = HttpConfig::new("https://api.wauldo.com")
    .with_header("X-RapidAPI-Key", "YOUR_RAPIDAPI_KEY")
    .with_header("X-RapidAPI-Host", "smart-rag-api.p.rapidapi.com");
let client = HttpClient::new(config)?;
```

Free tier (300 req/month): [RapidAPI](https://rapidapi.com/binnewzzin/api/smart-rag-api)

---

[Website](https://wauldo.com) | [Docs](https://wauldo.com/docs) | [Demo](https://wauldo.com/demo) | [Benchmarks](https://dev.to/wauldo/how-we-achieved-0-hallucination-rate-in-our-rag-api-with-benchmarks-4g54)

## Contributing

PRs welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for setup instructions and guidelines. Check the [good first issues](https://github.com/wauldo/wauldo-sdk-rust/labels/good%20first%20issue).

## License

MIT — see [LICENSE](./LICENSE)
