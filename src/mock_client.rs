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
        }
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
