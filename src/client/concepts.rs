//! Concept extraction methods for AgentClient

use serde_json::json;

use crate::error::{Error, Result};
use crate::types::*;

use super::AgentClient;

impl AgentClient {
    /// Extract concepts from text or code
    pub async fn extract_concepts(
        &mut self,
        text: &str,
        source_type: SourceType,
    ) -> Result<ConceptResult> {
        if text.trim().is_empty() {
            return Err(Error::validation_field("Text cannot be empty", "text"));
        }

        let source_type_str = match source_type {
            SourceType::Text => "text",
            SourceType::Code => "code",
        };

        let content = self
            .call_tool(
                "extract_concepts",
                json!({
                    "text": text,
                    "source_type": source_type_str
                }),
            )
            .await?;

        Ok(self.parse_concept_result(&content, source_type))
    }

    pub(crate) fn parse_concept_result(
        &self,
        content: &str,
        source_type: SourceType,
    ) -> ConceptResult {
        // Try JSON first (structured output from server v0.2+)
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(concepts_arr) = data.get("concepts").and_then(|c| c.as_array()) {
                let concepts = concepts_arr
                    .iter()
                    .map(|c| Concept {
                        name: c["name"].as_str().unwrap_or("").to_string(),
                        concept_type: c["concept_type"].as_str().unwrap_or("Entity").to_string(),
                        weight: c["weight"].as_f64().unwrap_or(0.8),
                        description: c["description"].as_str().map(String::from),
                    })
                    .collect();
                return ConceptResult {
                    concepts,
                    source_type,
                    raw_content: content.to_string(),
                };
            }
        }

        // Fallback: markdown heuristic parser (server v0.1.x)
        let mut concepts = Vec::new();

        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("- ") {
                let name = rest.split(':').next().unwrap_or("").trim();
                if !name.is_empty() {
                    concepts.push(Concept {
                        name: name.to_string(),
                        concept_type: "Entity".to_string(),
                        weight: 0.8,
                        description: None,
                    });
                }
            }
        }

        ConceptResult {
            concepts,
            source_type,
            raw_content: content.to_string(),
        }
    }
}
