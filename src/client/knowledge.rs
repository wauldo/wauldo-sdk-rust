//! Knowledge graph methods for AgentClient

use serde_json::json;

use crate::error::{Error, Result};
use crate::types::*;

use super::parsers::{parse_graph_nodes, parse_graph_stats};
use super::AgentClient;

impl AgentClient {
    /// Search knowledge graph
    pub async fn search_knowledge(
        &mut self,
        query: &str,
        limit: usize,
    ) -> Result<KnowledgeGraphResult> {
        if query.trim().is_empty() {
            return Err(Error::validation_field("Query cannot be empty", "query"));
        }

        let result = self
            .call_tool(
                "query_knowledge_graph",
                json!({
                    "operation": "search",
                    "query": query,
                    "limit": limit
                }),
            )
            .await?;

        Ok(KnowledgeGraphResult {
            operation: "search".to_string(),
            nodes: parse_graph_nodes(&result),
            stats: parse_graph_stats(&result),
            raw_content: result,
        })
    }

    /// Add to knowledge graph
    pub async fn add_to_knowledge(&mut self, text: &str) -> Result<KnowledgeGraphResult> {
        if text.trim().is_empty() {
            return Err(Error::validation_field("Text cannot be empty", "text"));
        }

        let result = self
            .call_tool(
                "query_knowledge_graph",
                json!({
                    "operation": "add",
                    "text": text
                }),
            )
            .await?;

        Ok(KnowledgeGraphResult {
            operation: "add".to_string(),
            nodes: parse_graph_nodes(&result),
            stats: parse_graph_stats(&result),
            raw_content: result,
        })
    }

    /// Get knowledge graph stats
    pub async fn knowledge_stats(&mut self) -> Result<KnowledgeGraphResult> {
        let result = self
            .call_tool("query_knowledge_graph", json!({ "operation": "stats" }))
            .await?;

        Ok(KnowledgeGraphResult {
            operation: "stats".to_string(),
            nodes: parse_graph_nodes(&result),
            stats: parse_graph_stats(&result),
            raw_content: result,
        })
    }
}
