//! Wauldo Rust SDK
//!
//! Provides two client interfaces:
//! - `AgentClient` — MCP server client (stdio JSON-RPC) for reasoning, planning, tools
//! - `HttpClient` — REST API client (OpenAI-compatible) for chat, embeddings, RAG, orchestrator
//!
//! # Quick Start (HTTP API)
//!
//! ```rust,no_run
//! use wauldo::{HttpClient, ChatRequest, ChatMessage, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = HttpClient::localhost()?;
//!
//!     let models = client.list_models().await?;
//!     println!("Models: {:?}", models.data.iter().map(|m| &m.id).collect::<Vec<_>>());
//!
//!     let req = ChatRequest::new("qwen2.5:7b", vec![ChatMessage::user("Hello!")]);
//!     let resp = client.chat(req).await?;
//!     println!("{}", resp.choices[0].message.content.as_deref().unwrap_or(""));
//!
//!     Ok(())
//! }
//! ```

mod client;
pub mod conversation;
mod error;
pub mod http_client;
pub mod http_config;
mod http_request;
pub mod http_types;
pub mod mock_client;
mod retry;
mod sse_parser;
mod transport;
mod types;

pub use client::AgentClient;
pub use conversation::Conversation;
pub use error::{Error, Result};
pub use http_client::HttpClient;
pub use http_config::HttpConfig;
pub use http_types::*;
pub use mock_client::MockHttpClient;
pub use types::*;
