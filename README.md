# Wauldo Rust SDK

[![Crates.io](https://img.shields.io/crates/v/wauldo.svg)](https://crates.io/crates/wauldo)
[![Downloads](https://img.shields.io/crates/d/wauldo.svg)](https://crates.io/crates/wauldo)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)

> **Verified AI answers from your documents.** Every response includes source citations, confidence scores, and an audit trail — or we don't answer at all.

Official Rust SDK for the [Wauldo API](https://wauldo.com) — the AI inference layer with smart model routing and zero hallucinations.

## Why Wauldo?

- **Zero hallucinations** — every answer is verified against source documents
- **Smart model routing** — auto-selects the cheapest model that meets quality (save 40-80% on AI costs)
- **One API, 7+ providers** — OpenAI, Anthropic, Google, Qwen, Meta, Mistral, DeepSeek with automatic fallback
- **OpenAI-compatible** — swap your `base_url`, keep your existing code
- **Full audit trail** — confidence score, grounded status, model used, latency on every response

## Quick Start

```rust
use wauldo::{HttpClient, HttpConfig, ChatRequest, ChatMessage, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::new(
        HttpConfig::new("https://api.wauldo.com").with_api_key("YOUR_API_KEY"),
    )?;

    let req = ChatRequest::new("auto", vec![ChatMessage::user("What is Rust?")]);
    let resp = client.chat(req).await?;
    println!("{}", resp.content());
    Ok(())
}
```

## Installation

```toml
[dependencies]
wauldo = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**Requirements:** Rust 1.70+

## Features

### Chat Completions

```rust
let req = ChatRequest::new("auto", vec![
    ChatMessage::system("You are a helpful assistant."),
    ChatMessage::user("Explain ownership in Rust"),
]);
let resp = client.chat(req).await?;
println!("{}", resp.content());
```

### RAG — Upload & Query

```rust
// Upload a document
let upload = client.rag_upload("Contract text here...", Some("contract.txt".into())).await?;
println!("Indexed {} chunks", upload.chunks_count);

// Query with verified answer
let result = client.rag_query("What are the payment terms?", None).await?;
println!("Answer: {}", result.answer);
println!("Confidence: {:.0}%", result.confidence() * 100.0);
println!("Grounded: {}", result.grounded());
for source in &result.sources {
    println!("  Source ({}%): {}", (source.score * 100.0) as u32, source.content);
}
```

### Streaming (SSE)

```rust
let req = ChatRequest::new("auto", vec![ChatMessage::user("Hello!")]);
let mut rx = client.chat_stream(req).await?;
while let Some(chunk) = rx.recv().await {
    print!("{}", chunk.unwrap_or_default());
}
```

### Conversation Helper

```rust
let mut conv = client.conversation()
    .with_system("You are an expert on Rust programming.")
    .with_model("auto");
let reply = conv.say("What is the borrow checker?").await?;
let follow_up = conv.say("Give me an example").await?;
```

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

## RapidAPI

```rust
let config = HttpConfig::new("https://api.wauldo.com")
    .with_header("X-RapidAPI-Key", "YOUR_RAPIDAPI_KEY")
    .with_header("X-RapidAPI-Host", "smart-rag-api.p.rapidapi.com");
let client = HttpClient::new(config)?;
```

Get your free API key (300 req/month): [RapidAPI](https://rapidapi.com/binnewzzin/api/smart-rag-api)

## Links

- [Website](https://wauldo.com)
- [Documentation](https://wauldo.com/docs)
- [Live Demo](https://api.wauldo.com/demo)
- [Cost Calculator](https://wauldo.com/calculator)
- [Status](https://wauldo.com/status)

## Contributing

Found a bug? Have a feature request? [Open an issue](https://github.com/wauldo/wauldo-sdk-rust/issues).

## License

MIT — see [LICENSE](./LICENSE)
