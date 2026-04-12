//! Transport layer for MCP communication

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::error::{Error, Result};

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

/// Stdio transport for MCP server communication
pub struct StdioTransport {
    server_path: Option<String>,
    timeout_ms: u64,
    process: Mutex<Option<Child>>,
    reader: Mutex<Option<BufReader<tokio::process::ChildStdout>>>,
    request_id: AtomicU64,
}

impl StdioTransport {
    /// Create new transport
    pub fn new(server_path: Option<String>, timeout_ms: u64) -> Self {
        Self {
            server_path,
            timeout_ms,
            process: Mutex::new(None),
            reader: Mutex::new(None),
            request_id: AtomicU64::new(0),
        }
    }

    /// Find MCP server binary
    fn find_server(&self) -> Result<PathBuf> {
        let search_paths = [
            std::env::current_dir()?.join("target/release/wauldo-mcp"),
            std::env::current_dir()?.join("target/debug/wauldo-mcp"),
            std::env::current_dir()?.join("../target/release/wauldo-mcp"),
            std::env::var("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_default()
                .join(".cargo/bin/wauldo-mcp"),
        ];

        for path in &search_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        Err(Error::connection(
            "MCP server binary not found. Please provide server_path or install with 'cargo install'."
        ))
    }

    /// Get server path
    fn get_server_path(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.server_path {
            Ok(PathBuf::from(path))
        } else {
            self.find_server()
        }
    }

    /// Connect to MCP server
    pub async fn connect(&self) -> Result<()> {
        // Check if already connected (quick check with process lock)
        {
            let guard = self.process.lock().await;
            if guard.is_some() {
                return Ok(());
            }
        }

        let server_path = self.get_server_path()?;

        let mut child = Command::new(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::connection(format!("Failed to start server: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::connection("No stdout from child process"))?;

        // Lock ordering: reader → process (consistent with disconnect())
        {
            let mut reader_guard = self.reader.lock().await;
            *reader_guard = Some(BufReader::new(stdout));
        }
        {
            let mut process_guard = self.process.lock().await;
            *process_guard = Some(child);
        }

        // Initialize MCP connection — kill process on failure
        if let Err(e) = self.initialize().await {
            self.disconnect().await;
            return Err(e);
        }

        Ok(())
    }

    /// Disconnect from server
    pub async fn disconnect(&self) {
        let mut reader_guard = self.reader.lock().await;
        *reader_guard = None;
        drop(reader_guard);

        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
        }
    }

    /// Send initialize request
    async fn initialize(&self) -> Result<()> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "wauldo-rust",
                "version": "0.1.0"
            }
        });

        self.request("initialize", Some(params)).await?;
        Ok(())
    }

    /// Send JSON-RPC request
    pub async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut process_guard = self.process.lock().await;
        let process = process_guard
            .as_mut()
            .ok_or_else(|| Error::connection("Not connected"))?;

        let id = self.request_id.fetch_add(1, Ordering::SeqCst) + 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        let request_data = serde_json::to_string(&request)? + "\n";

        // Write request
        let stdin = process
            .stdin
            .as_mut()
            .ok_or_else(|| Error::connection("No stdin"))?;
        stdin.write_all(request_data.as_bytes()).await?;
        stdin.flush().await?;

        // Read response from the persistent BufReader
        drop(process_guard); // release process lock before acquiring reader lock
        let mut reader_guard = self.reader.lock().await;
        let reader = reader_guard
            .as_mut()
            .ok_or_else(|| Error::connection("No reader — not connected"))?;
        let mut line = String::new();

        let read_future = reader.read_line(&mut line);
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            read_future,
        )
        .await
        .map_err(|_| Error::Timeout(format!("Request timed out after {}ms", self.timeout_ms)))?
        .map_err(|e| Error::connection(format!("Read error: {}", e)))?;

        if result == 0 {
            return Err(Error::connection("Server closed connection"));
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)?;

        if let Some(error) = response.error {
            return Err(Error::Server {
                code: error.code,
                message: error.message,
                data: None,
            });
        }

        Ok(response.result.unwrap_or(serde_json::Value::Null))
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // Best-effort kill of the child process to avoid orphan/zombie processes
        if let Ok(mut guard) = self.process.try_lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.start_kill();
            }
        } else {
            tracing::warn!(
                "StdioTransport dropped while lock held — child process may be orphaned"
            );
        }
    }
}
