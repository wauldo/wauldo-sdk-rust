//! Error types for Wauldo SDK

use thiserror::Error;

/// Result type alias for SDK operations
pub type Result<T> = std::result::Result<T, Error>;

/// SDK error types
#[derive(Debug, Error)]
pub enum Error {
    /// Connection to MCP server failed
    #[error("Connection error: {0}")]
    Connection(String),

    /// Server returned an error response
    #[error("Server error [{code}]: {message}")]
    Server {
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    },

    /// Input validation failed
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// Operation timed out
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
        }
    }

    /// Create a validation error with field name
    pub fn validation_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection(message.into())
    }

    /// Create a server error
    pub fn server(code: i32, message: impl Into<String>) -> Self {
        Self::Server {
            code,
            message: message.into(),
            data: None,
        }
    }
}
