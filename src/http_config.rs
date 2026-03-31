//! Configuration types for the HTTP client

use crate::error::Error;
use std::collections::HashMap;

/// Configuration for HTTP client
#[derive(Clone)]
pub struct HttpConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    /// Maximum number of retries on transient failures (default: 3)
    pub max_retries: u32,
    /// Base backoff duration in milliseconds (default: 1000)
    pub retry_backoff_ms: u64,
    /// Extra headers added to every request (e.g. X-RapidAPI-Key)
    pub extra_headers: HashMap<String, String>,
    /// Called before each request with (method, path)
    pub on_request: Option<fn(&str, &str)>,
    /// Called after each response with (status_code, duration_ms).
    ///
    /// **Note**: On non-streaming paths, status is always `200` (success implies 2xx).
    /// The `chat_stream` path provides the real HTTP status code.
    pub on_response: Option<fn(u16, u64)>,
    /// Called on request error
    pub on_error: Option<fn(&Error)>,
}

impl std::fmt::Debug for HttpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .field("timeout_secs", &self.timeout_secs)
            .field("max_retries", &self.max_retries)
            .field("retry_backoff_ms", &self.retry_backoff_ms)
            .field(
                "extra_headers",
                &self.extra_headers.keys().collect::<Vec<_>>(),
            )
            .field("on_request", &self.on_request.map(|_| "fn"))
            .field("on_response", &self.on_response.map(|_| "fn"))
            .field("on_error", &self.on_error.map(|_| "fn"))
            .finish()
    }
}

impl HttpConfig {
    /// Create a config with a custom base URL
    ///
    /// # Example
    /// ```rust
    /// use wauldo::HttpConfig;
    /// let config = HttpConfig::new("https://wauldo-api.p.rapidapi.com");
    /// assert_eq!(config.base_url, "https://wauldo-api.p.rapidapi.com");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
    }

    /// Create a config pointing to localhost:3000 with sensible defaults
    ///
    /// # Example
    /// ```rust
    /// use wauldo::HttpConfig;
    /// let config = HttpConfig::localhost();
    /// assert_eq!(config.base_url, "http://localhost:3000");
    /// ```
    pub fn localhost() -> Self {
        Self::default()
    }

    /// Set the API key (sent as `Authorization: Bearer <key>`)
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Add a custom header to every request (e.g. `X-RapidAPI-Key`)
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers.insert(name.into(), value.into());
        self
    }

    /// Set the request timeout in seconds
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            api_key: None,
            timeout_secs: 120,
            max_retries: 3,
            retry_backoff_ms: 1000,
            extra_headers: HashMap::new(),
            on_request: None,
            on_response: None,
            on_error: None,
        }
    }
}
