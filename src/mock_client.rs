//! Mock HTTP client for deterministic testing

use crate::error::Result;
use crate::http_types::*;

/// Mock HTTP client that returns predefined responses
///
/// Use this for deterministic unit tests without a running server.
///
/// # Example
/// ```rust,no_run
/// use wauldo::MockHttpClient;
/// let mock = MockHttpClient::new();
/// ```
pub struct MockHttpClient {
    chat_response: Option<ChatResponse>,
    models: Option<ModelList>,
    rag_upload_response: Option<RagUploadResponse>,
    rag_query_response: Option<RagQueryResponse>,
    fact_check_response: Option<FactCheckResponse>,
    verify_citation_response: Option<VerifyCitationResponse>,
}

impl MockHttpClient {
    /// Create a new empty mock client with no preconfigured responses
    ///
    /// # Example
    /// ```rust
    /// use wauldo::MockHttpClient;
    /// let mock = MockHttpClient::new();
    /// ```
    pub fn new() -> Self {
        Self {
            chat_response: None,
            models: None,
            rag_upload_response: None,
            rag_query_response: None,
            fact_check_response: None,
            verify_citation_response: None,
        }
    }

    /// Create a mock client pre-loaded with realistic sample data for all endpoints.
    ///
    /// This is the easiest way to explore the SDK without a running server.
    ///
    /// # Example
    /// ```rust
    /// use wauldo::MockHttpClient;
    /// let mock = MockHttpClient::with_defaults();
    /// ```
    pub fn with_defaults() -> Self {
        Self::new()
            .with_chat_response(ChatResponse {
                id: "mock-chat-001".to_string(),
                object: "chat.completion".to_string(),
                created: 1700000000,
                model: "qwen2.5:7b".to_string(),
                choices: vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: "assistant".to_string(),
                        content: Some(
                            "Rust's ownership model ensures memory safety without a garbage \
                             collector by enforcing strict rules about how values are owned, \
                             borrowed, and moved at compile time."
                                .to_string(),
                        ),
                        name: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: Usage {
                    prompt_tokens: 24,
                    completion_tokens: 38,
                    total_tokens: 62,
                },
            })
            .with_models(vec![
                Model {
                    id: "qwen2.5:7b".to_string(),
                    object: "model".to_string(),
                    created: 1700000000,
                    owned_by: "wauldo".to_string(),
                },
                Model {
                    id: "llama-4-scout".to_string(),
                    object: "model".to_string(),
                    created: 1700000000,
                    owned_by: "wauldo".to_string(),
                },
            ])
            .with_rag_upload(RagUploadResponse {
                document_id: "doc-abc123".to_string(),
                chunks_count: 5,
            })
            .with_rag_query(RagQueryResponse {
                answer: "Returns are accepted within 60 days of purchase. A valid receipt \
                         is required. [1]"
                    .to_string(),
                sources: vec![
                    RagSource {
                        document_id: "doc-abc123".to_string(),
                        content: "Our refund policy allows returns within 60 days of purchase. \
                                  Customers must provide a valid receipt."
                            .to_string(),
                        score: 0.92,
                        chunk_id: Some("chunk-001".to_string()),
                        metadata: None,
                    },
                    RagSource {
                        document_id: "doc-abc123".to_string(),
                        content: "Refunds are processed within 5 business days after approval."
                            .to_string(),
                        score: 0.78,
                        chunk_id: Some("chunk-002".to_string()),
                        metadata: None,
                    },
                ],
                audit: Some(RagAuditInfo {
                    confidence: 0.92,
                    retrieval_path: "BM25Only".to_string(),
                    sources_evaluated: 5,
                    sources_used: 2,
                    best_score: 0.92,
                    grounded: true,
                    confidence_label: "high".to_string(),
                    model: "qwen2.5:7b".to_string(),
                    latency_ms: 340,
                    candidates_found: Some(12),
                    candidates_after_tenant: Some(12),
                    candidates_after_score: Some(5),
                    query_type: Some("Factual".to_string()),
                }),
                confidence: None,
                grounded: None,
            })
            .with_fact_check(FactCheckResponse {
                verdict: "verified".to_string(),
                action: "allow".to_string(),
                hallucination_rate: 0.0,
                mode: "lexical".to_string(),
                total_claims: 2,
                supported_claims: 2,
                confidence: 0.85,
                claims: vec![
                    ClaimResult {
                        text: "Returns are accepted within 60 days.".to_string(),
                        claim_type: "factual".to_string(),
                        supported: true,
                        confidence: 0.90,
                        confidence_label: "high".to_string(),
                        verdict: "verified".to_string(),
                        action: "allow".to_string(),
                        reason: None,
                        evidence: Some("refund policy allows returns within 60 days".to_string()),
                    },
                    ClaimResult {
                        text: "A valid receipt is required.".to_string(),
                        claim_type: "factual".to_string(),
                        supported: true,
                        confidence: 0.82,
                        confidence_label: "high".to_string(),
                        verdict: "verified".to_string(),
                        action: "allow".to_string(),
                        reason: None,
                        evidence: Some("Customers must provide a valid receipt".to_string()),
                    },
                ],
                mode_warning: None,
                processing_time_ms: 1,
            })
            .with_verify_citation(VerifyCitationResponse {
                citation_ratio: 0.67,
                has_sufficient_citations: true,
                sentence_count: 3,
                citation_count: 2,
                uncited_sentences: vec![
                    "Refunds are processed quickly.".to_string(),
                ],
                citations: Some(vec![
                    CitationDetail {
                        citation: "[1]".to_string(),
                        source_name: "refund_policy.txt".to_string(),
                        is_valid: true,
                    },
                    CitationDetail {
                        citation: "[2]".to_string(),
                        source_name: "refund_policy.txt".to_string(),
                        is_valid: true,
                    },
                ]),
                phantom_count: Some(0),
                processing_time_ms: 1,
            })
    }

    /// Set the chat completion response returned by `chat()`
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{MockHttpClient, ChatResponse};
    /// # let resp: ChatResponse = todo!();
    /// let mock = MockHttpClient::new().with_chat_response(resp);
    /// ```
    pub fn with_chat_response(mut self, response: ChatResponse) -> Self {
        self.chat_response = Some(response);
        self
    }

    /// Set the models list returned by `list_models()`
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{MockHttpClient, Model};
    /// let mock = MockHttpClient::new().with_models(vec![]);
    /// ```
    pub fn with_models(mut self, models: Vec<Model>) -> Self {
        self.models = Some(ModelList {
            object: "list".to_string(),
            data: models,
        });
        self
    }

    /// Set the RAG upload response returned by `rag_upload()`
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{MockHttpClient, RagUploadResponse};
    /// # let resp: RagUploadResponse = todo!();
    /// let mock = MockHttpClient::new().with_rag_upload(resp);
    /// ```
    pub fn with_rag_upload(mut self, response: RagUploadResponse) -> Self {
        self.rag_upload_response = Some(response);
        self
    }

    /// Set the RAG query response returned by `rag_query()`
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{MockHttpClient, RagQueryResponse};
    /// # let resp: RagQueryResponse = todo!();
    /// let mock = MockHttpClient::new().with_rag_query(resp);
    /// ```
    pub fn with_rag_query(mut self, response: RagQueryResponse) -> Self {
        self.rag_query_response = Some(response);
        self
    }

    /// Set the fact-check response returned by `fact_check()`
    pub fn with_fact_check(mut self, response: FactCheckResponse) -> Self {
        self.fact_check_response = Some(response);
        self
    }

    /// Set the citation verification response returned by `verify_citation()`
    pub fn with_verify_citation(mut self, response: VerifyCitationResponse) -> Self {
        self.verify_citation_response = Some(response);
        self
    }

    /// List models (mocked) -- returns the value set via `with_models()`
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() -> wauldo::Result<()> {
    /// # use wauldo::MockHttpClient;
    /// let mock = MockHttpClient::new().with_models(vec![]);
    /// let models = mock.list_models().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_models(&self) -> Result<ModelList> {
        self.models
            .clone()
            .ok_or_else(|| crate::error::Error::connection("MockHttpClient: no models configured"))
    }

    /// Chat completion (mocked, non-streaming) -- ignores the request body
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() -> wauldo::Result<()> {
    /// # use wauldo::{MockHttpClient, ChatRequest, ChatMessage};
    /// # let resp: wauldo::ChatResponse = todo!();
    /// let mock = MockHttpClient::new().with_chat_response(resp);
    /// let req = ChatRequest::new("model", vec![ChatMessage::user("hi")]);
    /// let chat = mock.chat(req).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        self.chat_response.clone().ok_or_else(|| {
            crate::error::Error::connection("MockHttpClient: no chat response configured")
        })
    }

    /// Upload document for RAG (mocked) -- ignores the content
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() -> wauldo::Result<()> {
    /// # use wauldo::MockHttpClient;
    /// # let resp: wauldo::RagUploadResponse = todo!();
    /// let mock = MockHttpClient::new().with_rag_upload(resp);
    /// let upload = mock.rag_upload("text", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rag_upload(
        &self,
        _content: impl Into<String>,
        _filename: Option<String>,
    ) -> Result<RagUploadResponse> {
        self.rag_upload_response.clone().ok_or_else(|| {
            crate::error::Error::connection("MockHttpClient: no rag_upload response configured")
        })
    }

    /// Query RAG (mocked) -- ignores the query string
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() -> wauldo::Result<()> {
    /// # use wauldo::MockHttpClient;
    /// # let resp: wauldo::RagQueryResponse = todo!();
    /// let mock = MockHttpClient::new().with_rag_query(resp);
    /// let result = mock.rag_query("search term", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rag_query(
        &self,
        _query: impl Into<String>,
        _top_k: Option<usize>,
    ) -> Result<RagQueryResponse> {
        self.rag_query_response.clone().ok_or_else(|| {
            crate::error::Error::connection("MockHttpClient: no rag_query response configured")
        })
    }

    /// Query RAG with debug mode (mocked) -- same as rag_query
    pub async fn rag_query_debug(
        &self,
        query: impl Into<String>,
        top_k: Option<usize>,
    ) -> Result<RagQueryResponse> {
        self.rag_query(query, top_k).await
    }

    /// Chat completion with per-request timeout (mocked) -- timeout is ignored
    pub async fn chat_with_timeout(
        &self,
        request: ChatRequest,
        _timeout_ms: Option<u64>,
    ) -> Result<ChatResponse> {
        self.chat(request).await
    }

    /// Upload with per-request timeout (mocked) -- timeout is ignored
    pub async fn rag_upload_with_timeout(
        &self,
        content: impl Into<String>,
        filename: Option<String>,
        _timeout_ms: Option<u64>,
    ) -> Result<RagUploadResponse> {
        self.rag_upload(content, filename).await
    }

    /// Generate embeddings (mocked) -- returns a dummy embedding vector
    pub async fn embeddings(
        &self,
        _input: EmbeddingInput,
        _model: impl Into<String>,
    ) -> Result<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            data: vec![EmbeddingData {
                embedding: vec![0.1, 0.2, 0.3],
                index: 0,
            }],
            model: "mock-model".to_string(),
            usage: EmbeddingUsage {
                prompt_tokens: 5,
                total_tokens: 5,
            },
        })
    }

    /// Execute orchestrator (mocked) -- returns a fixed output
    pub async fn orchestrate(&self, _prompt: impl Into<String>) -> Result<OrchestratorResponse> {
        Ok(OrchestratorResponse {
            final_output: "Mock orchestration result".to_string(),
        })
    }

    /// Execute parallel swarm (mocked) -- returns a fixed output
    pub async fn orchestrate_parallel(
        &self,
        _prompt: impl Into<String>,
    ) -> Result<OrchestratorResponse> {
        Ok(OrchestratorResponse {
            final_output: "Mock parallel result".to_string(),
        })
    }

    // ── Insights & Analytics (mocked) ─────────────────────────────────

    /// GET /v1/insights (mocked) -- returns dummy ROI metrics
    pub async fn get_insights(&self) -> Result<InsightsResponse> {
        Ok(InsightsResponse {
            tig_key: "mock-key".to_string(),
            total_requests: 100,
            intelligence_requests: 80,
            fallback_requests: 20,
            tokens: InsightsTokenStats {
                baseline_total: 50000,
                real_total: 35000,
                saved_total: 15000,
                saved_percent_avg: 30.0,
            },
            cost: InsightsCostStats {
                estimated_usd_saved: 1.25,
            },
        })
    }

    /// GET /v1/analytics (mocked) -- returns dummy usage analytics
    pub async fn get_analytics(&self, _minutes: Option<u64>) -> Result<AnalyticsResponse> {
        Ok(AnalyticsResponse {
            cache: CacheMetrics {
                total_requests: 100,
                cache_hit_rate: 0.45,
                avg_latency_ms: 120.0,
                p95_latency_ms: 350.0,
            },
            tokens: TokenSavings {
                total_baseline: 50000,
                total_real: 35000,
                total_saved: 15000,
                avg_savings_percent: 30.0,
            },
            uptime_secs: 86400,
        })
    }

    /// GET /v1/analytics/traffic (mocked) -- returns dummy traffic summary
    pub async fn get_analytics_traffic(&self) -> Result<TrafficSummary> {
        Ok(TrafficSummary {
            total_requests_today: 250,
            total_tokens_today: 125000,
            top_tenants: vec![TenantTraffic {
                tenant_id: "mock-tenant".to_string(),
                requests_today: 250,
                tokens_used: 125000,
                success_rate: 0.98,
                avg_latency_ms: 150,
            }],
            error_rate: 0.02,
            avg_latency_ms: 150,
            p95_latency_ms: 400,
            uptime_secs: 86400,
        })
    }

    /// Fact-check (mocked) -- returns the value set via `with_fact_check()`
    pub async fn fact_check(&self, _request: FactCheckRequest) -> Result<FactCheckResponse> {
        self.fact_check_response.clone().ok_or_else(|| {
            crate::error::Error::connection("MockHttpClient: no fact_check response configured")
        })
    }

    /// Verify citation coverage (mocked) -- returns the value set via `with_verify_citation()`
    pub async fn verify_citation(
        &self,
        _request: VerifyCitationRequest,
    ) -> Result<VerifyCitationResponse> {
        self.verify_citation_response.clone().ok_or_else(|| {
            crate::error::Error::connection(
                "MockHttpClient: no verify_citation response configured",
            )
        })
    }

    /// Guard (mocked) -- returns a safe GuardResponse
    pub async fn guard(
        &self,
        text: impl Into<String>,
        source_context: impl Into<String>,
        mode: Option<&str>,
    ) -> Result<GuardResponse> {
        let t = text.into();
        let ctx = source_context.into();
        Ok(GuardResponse {
            verdict: "verified".to_string(),
            action: "allow".to_string(),
            hallucination_rate: 0.0,
            mode: mode.unwrap_or("lexical").to_string(),
            total_claims: 1,
            supported_claims: 1,
            confidence: 0.95,
            claims: vec![GuardClaim {
                text: t,
                claim_type: Some("Fact".to_string()),
                supported: true,
                confidence: 0.95,
                confidence_label: Some("high".to_string()),
                verdict: "verified".to_string(),
                action: "allow".to_string(),
                reason: None,
                evidence: Some(ctx),
            }],
            mode_warning: None,
            processing_time_ms: Some(0),
        })
    }

    /// Upload text into RAG and immediately query it (mocked) -- convenience one-shot
    pub async fn rag_ask(
        &self,
        _question: impl Into<String>,
        _text: impl Into<String>,
    ) -> Result<String> {
        let _ = self.rag_upload("mock", None).await?;
        Ok(self.rag_query("mock", None).await?.answer)
    }
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self::new()
    }
}
