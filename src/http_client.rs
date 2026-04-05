//! HTTP client for Wauldo REST API (OpenAI-compatible)

use crate::conversation::Conversation;
use crate::error::{Error, Result};
use crate::http_config::HttpConfig;
use crate::http_types::*;
use crate::retry::RetryConfig;
use crate::sse_parser::parse_sse_stream;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use tokio::sync::mpsc;

/// HTTP client for the Wauldo REST API
#[derive(Clone)]
pub struct HttpClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) retry_config: RetryConfig,
    pub(crate) on_request: Option<fn(&str, &str)>,
    pub(crate) on_response: Option<fn(u16, u64)>,
    pub(crate) on_error: Option<fn(&Error)>,
}

impl HttpClient {
    /// Create client with full configuration
    pub fn new(config: HttpConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "application/json".parse().expect("valid header"),
        );
        if let Some(key) = &config.api_key {
            let val = format!("Bearer {}", key)
                .parse()
                .map_err(|e| Error::connection(format!("Invalid API key: {}", e)))?;
            headers.insert(AUTHORIZATION, val);
        }
        for (name, value) in &config.extra_headers {
            let header_name: reqwest::header::HeaderName = name
                .parse()
                .map_err(|e| Error::connection(format!("Invalid header name '{}': {}", name, e)))?;
            let header_value = value.parse().map_err(|e| {
                Error::connection(format!("Invalid header value '{}': {}", value, e))
            })?;
            headers.insert(header_name, header_value);
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .default_headers(headers)
            .build()
            .map_err(|e| Error::connection(format!("HTTP client error: {}", e)))?;
        Ok(Self {
            client,
            base_url: config.base_url,
            retry_config: RetryConfig {
                max_retries: config.max_retries,
                backoff_ms: config.retry_backoff_ms,
            },
            on_request: config.on_request,
            on_response: config.on_response,
            on_error: config.on_error,
        })
    }

    /// Create client pointing to localhost:3000
    pub fn localhost() -> Result<Self> {
        Self::new(HttpConfig::default())
    }

    /// Create client with custom base URL
    pub fn with_url(base_url: impl Into<String>) -> Result<Self> {
        Self::new(HttpConfig {
            base_url: base_url.into(),
            ..Default::default()
        })
    }

    /// List available models -- GET /v1/models
    pub async fn list_models(&self) -> Result<ModelList> {
        self.get("/v1/models").await
    }

    /// Chat completion (non-streaming) -- POST /v1/chat/completions
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        if request.messages.is_empty() {
            return Err(Error::validation("messages cannot be empty"));
        }
        self.chat_with_timeout(request, None).await
    }

    /// Chat completion with an optional per-request timeout override
    pub async fn chat_with_timeout(
        &self,
        request: ChatRequest,
        timeout_ms: Option<u64>,
    ) -> Result<ChatResponse> {
        let mut req = request;
        req.stream = Some(false);
        self.post_with_timeout("/v1/chat/completions", &req, timeout_ms)
            .await
    }

    /// Chat completion with SSE streaming -- POST /v1/chat/completions
    pub async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let mut req = request;
        req.stream = Some(true);
        if let Some(hook) = self.on_request {
            hook("POST", "/v1/chat/completions");
        }
        let url = format!("{}/v1/chat/completions", self.base_url);
        let start = std::time::Instant::now();
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                let err = if e.is_timeout() {
                    Error::Timeout(format!("Request timed out: {}", e))
                } else {
                    Error::connection(format!("Request failed: {}", e))
                };
                if let Some(hook) = self.on_error {
                    hook(&err);
                }
                err
            })?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            let message = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v.get("error")?.get("message")?.as_str().map(String::from))
                .unwrap_or(body);
            let err = Error::server(status as i32, message);
            if let Some(hook) = self.on_error {
                hook(&err);
            }
            return Err(err);
        }
        if let Some(hook) = self.on_response {
            hook(status, start.elapsed().as_millis() as u64);
        }
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            if let Err(e) = parse_sse_stream(resp, tx).await {
                tracing::error!("SSE stream error: {}", e);
            }
        });
        Ok(rx)
    }

    /// Generate embeddings -- POST /v1/embeddings
    pub async fn embeddings(
        &self,
        input: EmbeddingInput,
        model: impl Into<String>,
    ) -> Result<EmbeddingResponse> {
        self.post(
            "/v1/embeddings",
            &EmbeddingRequest {
                input,
                model: model.into(),
            },
        )
        .await
    }

    /// Upload document for RAG -- POST /v1/upload
    pub async fn rag_upload(
        &self,
        content: impl Into<String>,
        filename: Option<String>,
    ) -> Result<RagUploadResponse> {
        self.rag_upload_with_timeout(content, filename, None).await
    }

    /// Upload document for RAG with an optional per-request timeout override
    pub async fn rag_upload_with_timeout(
        &self,
        content: impl Into<String>,
        filename: Option<String>,
        timeout_ms: Option<u64>,
    ) -> Result<RagUploadResponse> {
        self.post_with_timeout(
            "/v1/upload",
            &RagUploadRequest {
                content: content.into(),
                filename,
            },
            timeout_ms,
        )
        .await
    }

    /// Query RAG -- POST /v1/query
    pub async fn rag_query(
        &self,
        query: impl Into<String>,
        top_k: Option<usize>,
    ) -> Result<RagQueryResponse> {
        self.post(
            "/v1/query",
            &RagQueryRequest {
                query: query.into(),
                top_k,
                debug: None,
                stream: None,
                quality_mode: None,
            },
        )
        .await
    }

    /// Query RAG with debug mode — returns retrieval funnel diagnostics
    pub async fn rag_query_debug(
        &self,
        query: impl Into<String>,
        top_k: Option<usize>,
    ) -> Result<RagQueryResponse> {
        self.post(
            "/v1/query",
            &RagQueryRequest {
                query: query.into(),
                top_k,
                debug: Some(true),
                stream: None,
                quality_mode: None,
            },
        )
        .await
    }

    /// Execute orchestrator (best agent) -- POST /v1/orchestrator/execute
    pub async fn orchestrate(&self, prompt: impl Into<String>) -> Result<OrchestratorResponse> {
        self.post(
            "/v1/orchestrator/execute",
            &OrchestratorRequest {
                prompt: prompt.into(),
            },
        )
        .await
    }

    /// Execute parallel swarm (all specialists) -- POST /v1/orchestrator/parallel
    pub async fn orchestrate_parallel(
        &self,
        prompt: impl Into<String>,
    ) -> Result<OrchestratorResponse> {
        self.post(
            "/v1/orchestrator/parallel",
            &OrchestratorRequest {
                prompt: prompt.into(),
            },
        )
        .await
    }

    // ── Fact-Check ─────────────────────────────────────────────────────

    /// POST /v1/fact-check — Verify claims against source context
    pub async fn fact_check(&self, request: FactCheckRequest) -> Result<FactCheckResponse> {
        self.post("/v1/fact-check", &request).await
    }

    /// POST /v1/verify — Verify citation coverage in text
    pub async fn verify_citation(
        &self,
        request: VerifyCitationRequest,
    ) -> Result<VerifyCitationResponse> {
        self.post("/v1/verify", &request).await
    }

    // ── Insights & Analytics ──────────────────────────────────────────

    /// GET /v1/insights — ROI metrics for your API key
    pub async fn get_insights(&self) -> Result<InsightsResponse> {
        self.get("/v1/insights").await
    }

    /// GET /v1/analytics — Usage analytics and cache performance
    pub async fn get_analytics(&self, minutes: Option<u64>) -> Result<AnalyticsResponse> {
        let path = match minutes {
            Some(m) => format!("/v1/analytics?minutes={}", m),
            None => "/v1/analytics".to_string(),
        };
        self.get(&path).await
    }

    /// GET /v1/analytics/traffic — Per-tenant traffic monitoring
    pub async fn get_analytics_traffic(&self) -> Result<TrafficSummary> {
        self.get("/v1/analytics/traffic").await
    }

    /// Create a stateful conversation helper using this client
    pub fn conversation(&self) -> Conversation {
        Conversation::new(self.clone())
    }

    /// Upload text into RAG and immediately query it -- convenience one-shot
    pub async fn rag_ask(&self, question: &str, text: &str) -> Result<String> {
        self.rag_upload(text, None).await?;
        Ok(self.rag_query(question, None).await?.answer)
    }
}
