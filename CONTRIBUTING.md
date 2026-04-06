# Contributing to Wauldo Rust SDK

Thanks for your interest in contributing! This guide will get you set up in under 5 minutes.

## Setup

```bash
git clone https://github.com/wauldo/wauldo-sdk-rust.git
cd wauldo-sdk-rust
cargo build
cargo test
```

Requirements: Rust 1.70+ (install via [rustup](https://rustup.rs))

## Testing without a server

The SDK includes `MockHttpClient` for deterministic testing without a running Wauldo server.

```rust
use wauldo::MockHttpClient;

#[tokio::test]
async fn test_my_feature() {
    let client = MockHttpClient::with_defaults();
    let result = client.rag_query("test query", None).await.unwrap();
    assert!(!result.answer.is_empty());
}
```

`MockHttpClient::with_defaults()` provides realistic sample data for every endpoint. You can also configure specific responses with the builder pattern:

```rust
let client = MockHttpClient::new()
    .with_rag_query(my_custom_response)
    .with_fact_check(my_custom_fc_response);
```

Run examples locally:

```bash
cargo run --example quickstart
cargo run --example analytics_demo
```

## Code style

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets -- -D warnings

# Run both before committing
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

## Making a PR

1. **One PR per feature** -- keep changes focused and reviewable
2. **Add tests** -- use `MockHttpClient` for unit tests, no server required
3. **Update README** -- if you add a public API method, show it in the README
4. **Run the checks** -- `cargo fmt`, `cargo clippy`, `cargo test` must all pass
5. **Write a clear commit message** -- follow `<type>(<scope>): <subject>` (e.g. `feat(mock): add verify_citation method`)

## What to contribute

Check the [open issues](https://github.com/wauldo/wauldo-sdk-rust/issues) for ideas. Good first contributions:

- New examples showing real-world usage patterns
- Additional mock responses for edge cases
- Documentation improvements
- Bug fixes with regression tests

## Project structure

```
src/
  lib.rs           -- public exports
  http_client.rs   -- real HTTP client (reqwest-based)
  mock_client.rs   -- mock client for testing
  http_types.rs    -- all request/response types
  http_config.rs   -- client configuration
  error.rs         -- error types
  conversation.rs  -- stateful conversation helper
examples/
  quickstart.rs    -- full SDK walkthrough (no server needed)
  analytics_demo.rs -- insights + analytics + traffic
  basic_chat.rs    -- chat completion
  rag_workflow.rs  -- document upload + query
  streaming_chat.rs -- SSE streaming
```

## Questions?

Open an issue or start a discussion on [GitHub](https://github.com/wauldo/wauldo-sdk-rust/issues).
