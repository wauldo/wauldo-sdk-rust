//! Internal request execution with hooks and retry logic

use crate::error::{Error, Result};
use crate::http_client::HttpClient;
use crate::retry::request_with_retry;

impl HttpClient {
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        tracing::debug!("SDK request: GET {}", path);
        if let Some(hook) = self.on_request {
            hook("GET", path);
        }
        let url = format!("{}{}", self.base_url, path);
        let start = std::time::Instant::now();
        let result = request_with_retry(&self.client, &self.retry_config, || {
            self.client.get(&url).build()
        })
        .await;
        self.fire_response_hooks(&result, start);
        result
    }

    pub(crate) async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T> {
        self.post_with_timeout(path, body, None).await
    }

    pub(crate) async fn post_with_timeout<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
        timeout_ms: Option<u64>,
    ) -> Result<T> {
        tracing::debug!("SDK request: POST {}", path);
        if let Some(hook) = self.on_request {
            hook("POST", path);
        }
        let url = format!("{}{}", self.base_url, path);
        let json = serde_json::to_value(body)
            .map_err(|e| Error::connection(format!("Serialize error: {}", e)))?;
        let start = std::time::Instant::now();
        let result = match timeout_ms {
            Some(ms) => {
                request_with_retry(&self.client, &self.retry_config, || {
                    self.client
                        .post(&url)
                        .timeout(std::time::Duration::from_millis(ms))
                        .json(&json)
                        .build()
                })
                .await
            }
            None => {
                request_with_retry(&self.client, &self.retry_config, || {
                    self.client.post(&url).json(&json).build()
                })
                .await
            }
        };
        self.fire_response_hooks(&result, start);
        result
    }

    pub(crate) fn fire_response_hooks<T>(&self, result: &Result<T>, start: std::time::Instant) {
        let ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match result {
            Ok(_) => {
                tracing::debug!("SDK response: OK in {}ms", ms);
                if let Some(hook) = self.on_response {
                    // Note: actual HTTP status not available from request_with_retry —
                    // success implies 2xx; chat_stream path provides real status.
                    hook(200, ms);
                }
            }
            Err(e) => {
                tracing::debug!("SDK response: error in {}ms -- {}", ms, e);
                if let Some(hook) = self.on_error {
                    hook(e);
                }
            }
        }
    }
}
