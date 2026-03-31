//! Document chunking and retrieval methods for AgentClient

use serde_json::json;

use crate::error::{Error, Result};
use crate::types::*;

use super::parsers::{parse_chunks, parse_retrieval_results};
use super::AgentClient;

impl AgentClient {
    /// Chunk a document
    pub async fn chunk_document(
        &mut self,
        content: &str,
        chunk_size: usize,
    ) -> Result<ChunkResult> {
        if content.trim().is_empty() {
            return Err(Error::validation_field(
                "Content cannot be empty",
                "content",
            ));
        }

        let result = self
            .call_tool(
                "manage_long_context",
                json!({
                    "operation": "chunk",
                    "content": content,
                    "chunk_size": chunk_size
                }),
            )
            .await?;

        let chunks = parse_chunks(&result);
        let total_chunks = chunks.len();
        Ok(ChunkResult {
            chunks,
            total_chunks,
            raw_content: result,
        })
    }

    /// Retrieve context for a query
    pub async fn retrieve_context(&mut self, query: &str, top_k: usize) -> Result<RetrievalResult> {
        if query.trim().is_empty() {
            return Err(Error::validation_field("Query cannot be empty", "query"));
        }

        let result = self
            .call_tool(
                "manage_long_context",
                json!({
                    "operation": "retrieve",
                    "query": query,
                    "top_k": top_k
                }),
            )
            .await?;

        let results = parse_retrieval_results(&result);
        Ok(RetrievalResult {
            query: query.to_string(),
            results,
            raw_content: result,
        })
    }

    /// Summarize content
    pub async fn summarize(&mut self, content: &str) -> Result<String> {
        if content.trim().is_empty() {
            return Err(Error::validation_field(
                "Content cannot be empty",
                "content",
            ));
        }

        self.call_tool(
            "manage_long_context",
            json!({
                "operation": "summarize",
                "content": content
            }),
        )
        .await
    }
}
