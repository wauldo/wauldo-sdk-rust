//! Type definitions for Wauldo SDK

use serde::{Deserialize, Serialize};

/// Reasoning result from Tree-of-Thought
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    /// The original problem
    pub problem: String,
    /// The derived solution
    pub solution: String,
    /// Full thought tree output
    pub thought_tree: String,
    /// Depth of reasoning
    pub depth: usize,
    /// Number of branches
    pub branches: usize,
    /// Raw content from server
    pub raw_content: String,
}

/// Options for reasoning
#[derive(Debug, Clone, Default)]
pub struct ReasoningOptions {
    /// Depth of the thought tree (1-10)
    pub depth: Option<usize>,
    /// Number of branches at each level (1-10)
    pub branches: Option<usize>,
}

impl ReasoningOptions {
    /// Create new options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set depth
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Set branches
    pub fn branches(mut self, branches: usize) -> Self {
        self.branches = Some(branches);
        self
    }
}

/// Source type for concept extraction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    /// Plain text
    Text,
    /// Source code
    Code,
}

/// A single extracted concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Concept name
    pub name: String,
    /// Type of concept
    pub concept_type: String,
    /// Confidence weight (0.0-1.0)
    pub weight: f64,
    /// Optional description
    pub description: Option<String>,
}

/// Result from concept extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptResult {
    /// Extracted concepts
    pub concepts: Vec<Concept>,
    /// Source type used
    pub source_type: SourceType,
    /// Raw content from server
    pub raw_content: String,
}

/// A text chunk from document processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk identifier
    pub id: String,
    /// Chunk content
    pub content: String,
    /// Position in original document
    pub position: usize,
    /// Priority level
    pub priority: String,
}

/// Result from document chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResult {
    /// Generated chunks
    pub chunks: Vec<Chunk>,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Raw content from server
    pub raw_content: String,
}

/// Result from context retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// Query used
    pub query: String,
    /// Retrieved chunks
    pub results: Vec<Chunk>,
    /// Raw content from server
    pub raw_content: String,
}

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Node identifier
    pub id: String,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: String,
    /// Weight/importance
    pub weight: f64,
}

/// Result from knowledge graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphResult {
    /// Operation performed
    pub operation: String,
    /// Result nodes
    pub nodes: Vec<GraphNode>,
    /// Graph statistics
    pub stats: Option<serde_json::Value>,
    /// Raw content from server
    pub raw_content: String,
}

/// Detail level for task planning
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetailLevel {
    /// Minimal detail
    Brief,
    /// Standard detail
    #[default]
    Normal,
    /// Maximum detail
    Detailed,
}

/// A single step in a task plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step number
    pub number: usize,
    /// Step title
    pub title: String,
    /// Step description
    pub description: String,
    /// Priority level
    pub priority: String,
    /// Effort estimate
    pub effort: String,
    /// Dependencies on other steps
    pub dependencies: Vec<String>,
}

/// Result from task planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResult {
    /// Original task
    pub task: String,
    /// Task category
    pub category: String,
    /// Plan steps
    pub steps: Vec<PlanStep>,
    /// Total effort estimate
    pub total_effort: String,
    /// Raw content from server
    pub raw_content: String,
}

/// Options for task planning
#[derive(Debug, Clone, Default)]
pub struct PlanOptions {
    /// Additional context
    pub context: Option<String>,
    /// Maximum steps (1-20)
    pub max_steps: Option<usize>,
    /// Detail level
    pub detail_level: Option<DetailLevel>,
}

impl PlanOptions {
    /// Create new options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set context
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set max steps
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = Some(max_steps);
        self
    }

    /// Set detail level
    pub fn detail_level(mut self, level: DetailLevel) -> Self {
        self.detail_level = Some(level);
        self
    }
}

/// Tool definition from MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: serde_json::Value,
}

/// Client configuration options
#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// Path to MCP server binary
    pub server_path: Option<String>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Auto-connect on first operation
    pub auto_connect: bool,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            server_path: None,
            timeout_ms: 30000,
            auto_connect: true,
        }
    }
}

impl ClientOptions {
    /// Create new options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set server path
    pub fn server_path(mut self, path: impl Into<String>) -> Self {
        self.server_path = Some(path.into());
        self
    }

    /// Set timeout
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// Set auto-connect
    pub fn auto_connect(mut self, auto: bool) -> Self {
        self.auto_connect = auto;
        self
    }
}
