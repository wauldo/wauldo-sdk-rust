//! Wauldo client — struct definition, constructor, tool primitives

mod chunking;
mod concepts;
mod knowledge;
mod parsers;
mod planning;
mod reasoning;

use serde_json::json;

use crate::error::{Error, Result};
use crate::transport::StdioTransport;
use crate::types::*;

/// Client for Wauldo MCP Server
///
/// # Example
///
/// ```rust,no_run
/// use wauldo::{AgentClient, ReasoningOptions};
///
/// #[tokio::main]
/// async fn main() -> wauldo::Result<()> {
///     let mut client = AgentClient::new().connect().await?;
///
///     let result = client.reason("How to optimize?").await?;
///     println!("{}", result.solution);
///
///     client.disconnect().await;
///     Ok(())
/// }
/// ```
pub struct AgentClient {
    pub(crate) transport: StdioTransport,
    pub(crate) auto_connect: bool,
    pub(crate) connected: bool,
}

impl AgentClient {
    /// Create a new client with default options
    pub fn new() -> Self {
        Self::with_options(ClientOptions::default())
    }

    /// Create a new client with custom options
    pub fn with_options(options: ClientOptions) -> Self {
        Self {
            transport: StdioTransport::new(options.server_path, options.timeout_ms),
            auto_connect: options.auto_connect,
            connected: false,
        }
    }

    /// Connect to MCP server
    pub async fn connect(mut self) -> Result<Self> {
        self.transport.connect().await?;
        self.connected = true;
        Ok(self)
    }

    /// Disconnect from MCP server
    pub async fn disconnect(&mut self) {
        self.transport.disconnect().await;
        self.connected = false;
    }

    /// Ensure client is connected
    pub(crate) async fn ensure_connected(&mut self) -> Result<()> {
        if !self.connected {
            if self.auto_connect {
                self.transport.connect().await?;
                self.connected = true;
            } else {
                return Err(Error::connection("Not connected. Call connect() first."));
            }
        }
        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>> {
        self.ensure_connected().await?;
        let result = self.transport.request("tools/list", None).await?;
        let tools = result
            .get("tools")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();
        Ok(tools)
    }

    /// Call a tool by name
    pub async fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> Result<String> {
        self.ensure_connected().await?;
        let result = self
            .transport
            .request(
                "tools/call",
                Some(json!({
                    "name": name,
                    "arguments": arguments
                })),
            )
            .await?;

        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }
}

/// Creates a disconnected client with default options.
/// Call `.connect().await?` before using MCP methods.
impl Default for AgentClient {
    fn default() -> Self {
        Self::new()
    }
}
