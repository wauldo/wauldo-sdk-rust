# Changelog

All notable changes to the Wauldo Rust SDK.

## [0.1.0] - 2026-03-16

### Added
- `HttpClient` — REST API client (OpenAI-compatible)
  - `chat()`, `chat_stream()`, `list_models()`, `embeddings()`
  - `rag_upload()`, `rag_query()`, `rag_ask()`
  - `orchestrate()`, `orchestrate_parallel()`
- `AgentClient` — MCP client (stdio JSON-RPC)
  - `reason()`, `extract_concepts()`, `plan_task()`
  - `chunk_document()`, `retrieve_context()`, `summarize()`
  - `search_knowledge()`, `add_to_knowledge()`
- `Conversation` — automatic chat history management
- `MockHttpClient` — offline testing without server
- Retry with exponential backoff (429/503/network errors)
- Structured logging via `tracing` crate
- Response validation with detailed error messages
- 30 unit tests + 22 doc tests
