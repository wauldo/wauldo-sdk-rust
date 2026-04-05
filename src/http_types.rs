//! HTTP API request/response types (OpenAI-compatible)

use serde::{Deserialize, Serialize};

// ── Chat Completions ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

impl ChatResponse {
    /// Get the text content of the first choice (None if no content)
    pub fn text(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.content.as_deref())
    }

    /// Get the text content or an empty string — convenience for display
    pub fn content(&self) -> String {
        self.text().unwrap_or("").to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

// ── Streaming ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub id: String,
    pub choices: Vec<ChatChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    pub content: Option<String>,
}

// ── Usage ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ── Models ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

// ── Embeddings ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    pub input: EmbeddingInput,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

// ── RAG ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct RagUploadRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagUploadResponse {
    pub document_id: String,
    pub chunks_count: usize,
}

/// Quality assessment of an uploaded document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentQuality {
    pub score: f32,
    pub label: String,
    pub word_count: usize,
    pub line_density: f32,
    pub avg_line_length: f32,
    pub paragraph_count: usize,
}

/// Response from POST /v1/upload-file (PDF, DOCX, text, image)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileResponse {
    pub document_id: String,
    pub chunks_count: usize,
    pub indexed_at: String,
    pub content_type: String,
    pub trace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<DocumentQuality>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RagQueryRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,
    /// Enable debug mode — returns retrieval funnel details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
    /// Enable SSE streaming (sources → token* → audit → \[DONE\])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Quality mode: "fast", "balanced", "premium"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagQueryResponse {
    pub answer: String,
    pub sources: Vec<RagSource>,
    /// Full audit trail — always present
    #[serde(default)]
    pub audit: Option<RagAuditInfo>,
    // Legacy flat fields (servers < v1.6.5 may return these at root level)
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub grounded: Option<bool>,
}

impl RagQueryResponse {
    /// Get confidence from audit (preferred) or legacy flat field
    pub fn confidence(&self) -> Option<f32> {
        self.audit.as_ref().map(|a| a.confidence).or(self.confidence)
    }

    /// Get grounded from audit (preferred) or legacy flat field
    pub fn grounded(&self) -> Option<bool> {
        self.audit.as_ref().map(|a| a.grounded).or(self.grounded)
    }
}

/// Audit trail for RAG responses — verification and accountability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagAuditInfo {
    pub confidence: f32,
    pub retrieval_path: String,
    pub sources_evaluated: usize,
    pub sources_used: usize,
    pub best_score: f32,
    pub grounded: bool,
    pub confidence_label: String,
    pub model: String,
    pub latency_ms: u64,
    /// Retrieval funnel diagnostics (v1.6.5+)
    #[serde(default)]
    pub candidates_found: Option<usize>,
    #[serde(default)]
    pub candidates_after_tenant: Option<usize>,
    #[serde(default)]
    pub candidates_after_score: Option<usize>,
    #[serde(default)]
    pub query_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSource {
    pub document_id: String,
    pub content: String,
    pub score: f32,
    #[serde(default)]
    pub chunk_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

// ── Orchestrator ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OrchestratorRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorResponse {
    pub final_output: String,
}

// ── Fact-Check ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FactCheckRequest {
    pub text: String,
    pub source_context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResult {
    pub text: String,
    pub claim_type: String,
    pub supported: bool,
    pub confidence: f64,
    pub confidence_label: String,
    pub verdict: String,
    pub action: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactCheckResponse {
    pub verdict: String,
    pub action: String,
    pub hallucination_rate: f64,
    pub mode: String,
    pub total_claims: usize,
    pub supported_claims: usize,
    pub confidence: f64,
    pub claims: Vec<ClaimResult>,
    #[serde(default)]
    pub mode_warning: Option<String>,
    pub processing_time_ms: u64,
}

impl FactCheckRequest {
    pub fn new(text: impl Into<String>, source_context: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            source_context: source_context.into(),
            mode: None,
        }
    }

    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }
}

// ── Citation Verify ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceChunk {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationDetail {
    pub citation: String,
    pub source_name: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyCitationRequest {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SourceChunk>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCitationResponse {
    pub citation_ratio: f64,
    pub has_sufficient_citations: bool,
    pub sentence_count: usize,
    pub citation_count: usize,
    pub uncited_sentences: Vec<String>,
    #[serde(default)]
    pub citations: Option<Vec<CitationDetail>>,
    #[serde(default)]
    pub phantom_count: Option<usize>,
    pub processing_time_ms: u64,
}

impl VerifyCitationRequest {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            sources: None,
            threshold: None,
        }
    }

    pub fn with_sources(mut self, sources: Vec<SourceChunk>) -> Self {
        self.sources = Some(sources);
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

// ── Builders ────────────────────────────────────────────────────────────

impl ChatRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stream: None,
            top_p: None,
            stop: None,
        }
    }

    /// Create a quick single-message chat request
    pub fn quick(model: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(model, vec![ChatMessage::user(message)])
    }
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: Some(content.into()),
            name: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: Some(content.into()),
            name: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: Some(content.into()),
            name: None,
        }
    }
}
