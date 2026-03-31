//! Exponential backoff retry for transient HTTP errors

use crate::error::{Error, Result};

/// Retry configuration
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 1000,
        }
    }
}

/// Execute an HTTP request with exponential backoff on transient errors
///
/// Retries on HTTP 429 (rate limit) and 503 (service unavailable), as well
/// as network-level failures. Respects the `Retry-After` header when present.
/// Non-retryable status codes (e.g. 400, 401, 404) fail immediately.
///
/// # Example
/// ```rust,no_run
/// # use wauldo::HttpClient;
/// // Retry is used internally by HttpClient; you do not call it directly.
/// let client = HttpClient::localhost().unwrap();
/// ```
pub async fn request_with_retry<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    config: &RetryConfig,
    build_request: impl Fn() -> std::result::Result<reqwest::Request, reqwest::Error>,
) -> Result<T> {
    let mut last_err = Error::connection("No attempts made");
    for attempt in 0..=config.max_retries {
        let request = build_request()
            .map_err(|e| Error::connection(format!("Build request error: {}", e)))?;
        match client.execute(request).await {
            Ok(resp) if resp.status().is_success() => {
                let status = resp.status().as_u16();
                let body = resp
                    .text()
                    .await
                    .map_err(|e| Error::connection(format!("Read body error: {}", e)))?;
                return serde_json::from_str::<T>(&body).map_err(|e| {
                    let preview = if body.len() > 200 {
                        &body[..200]
                    } else {
                        &body
                    };
                    Error::server(
                        status as i32,
                        format!("Invalid response: {} — body: {}", e, preview),
                    )
                });
            }
            Ok(resp) if is_retryable(resp.status()) => {
                let status = resp.status().as_u16();
                let retry_after = parse_retry_after(&resp);
                let body = resp.text().await.unwrap_or_else(|e| {
                    tracing::warn!("Failed to read error body: {}", e);
                    String::new()
                });
                last_err = Error::server(status as i32, body);
                if attempt < config.max_retries {
                    let delay = retry_after.unwrap_or(backoff_duration(config, attempt));
                    tracing::warn!(attempt, status, ?delay, "Retrying request");
                    tokio::time::sleep(delay).await;
                }
            }
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_else(|e| {
                    tracing::warn!("Failed to read error body: {}", e);
                    String::new()
                });
                return Err(Error::server(status as i32, body));
            }
            Err(e) => {
                last_err = if e.is_timeout() {
                    Error::Timeout(format!("Request timed out: {}", e))
                } else {
                    Error::connection(format!("Request failed: {}", e))
                };
                if attempt < config.max_retries {
                    let delay = backoff_duration(config, attempt);
                    tracing::warn!(attempt, %e, ?delay, "Retrying after network error");
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Err(last_err)
}

fn is_retryable(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504)
}

fn backoff_duration(config: &RetryConfig, attempt: u32) -> std::time::Duration {
    std::time::Duration::from_millis(config.backoff_ms.saturating_mul(2u64.saturating_pow(attempt.min(20))))
}

fn parse_retry_after(resp: &reqwest::Response) -> Option<std::time::Duration> {
    resp.headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .map(std::time::Duration::from_secs)
}
