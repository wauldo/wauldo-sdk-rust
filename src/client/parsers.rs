//! Shared JSON parsers for chunk, retrieval, and knowledge graph responses

use crate::types::{Chunk, GraphNode};

/// Parse chunk/result items from JSON response, trying `primary_key` first then `fallback_key`
fn parse_chunk_list(raw: &str, primary_key: &str, fallback_key: &str) -> Vec<Chunk> {
    let Ok(data) = serde_json::from_str::<serde_json::Value>(raw) else {
        return vec![];
    };
    let items = data
        .get(primary_key)
        .or_else(|| data.get(fallback_key))
        .and_then(|v| v.as_array());
    let Some(items) = items else {
        return vec![];
    };
    items
        .iter()
        .enumerate()
        .map(|(i, c)| Chunk {
            id: c
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            content: c
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            position: c
                .get("position")
                .and_then(|v| v.as_u64())
                .unwrap_or(i as u64) as usize,
            priority: c
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .to_string(),
        })
        .collect()
}

/// Parse chunk items — tries "chunks" first, then "results"
pub(super) fn parse_chunks(raw: &str) -> Vec<Chunk> {
    parse_chunk_list(raw, "chunks", "results")
}

/// Parse retrieval results — tries "results" first, then "chunks"
pub(super) fn parse_retrieval_results(raw: &str) -> Vec<Chunk> {
    parse_chunk_list(raw, "results", "chunks")
}

/// Parse knowledge graph nodes from JSON response
pub(super) fn parse_graph_nodes(raw: &str) -> Vec<GraphNode> {
    let Ok(data) = serde_json::from_str::<serde_json::Value>(raw) else {
        return vec![];
    };
    let items = data.get("nodes").and_then(|v| v.as_array());
    let Some(items) = items else {
        return vec![];
    };
    items
        .iter()
        .map(|n| GraphNode {
            id: n
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: n
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            node_type: n
                .get("node_type")
                .or_else(|| n.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            weight: n.get("weight").and_then(|v| v.as_f64()).unwrap_or(0.0),
        })
        .collect()
}

/// Parse graph stats from JSON response
pub(super) fn parse_graph_stats(raw: &str) -> Option<serde_json::Value> {
    let data = serde_json::from_str::<serde_json::Value>(raw).ok()?;
    data.get("stats").cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chunks_with_chunks_key() {
        let raw = r#"{"chunks":[{"id":"c1","content":"hello","position":0,"priority":"high"}]}"#;
        let chunks = parse_chunks(raw);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].id, "c1");
        assert_eq!(chunks[0].content, "hello");
        assert_eq!(chunks[0].priority, "high");
    }

    #[test]
    fn test_parse_chunks_falls_back_to_results_key() {
        let raw = r#"{"results":[{"id":"r1","content":"world"}]}"#;
        let chunks = parse_chunks(raw);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].id, "r1");
    }

    #[test]
    fn test_parse_retrieval_results_prefers_results_key() {
        let raw = r#"{"results":[{"id":"r1","content":"a"}],"chunks":[{"id":"c1","content":"b"}]}"#;
        let results = parse_retrieval_results(raw);
        assert_eq!(results[0].id, "r1");
    }

    #[test]
    fn test_parse_chunks_empty_json() {
        assert!(parse_chunks("{}").is_empty());
    }

    #[test]
    fn test_parse_chunks_invalid_json() {
        assert!(parse_chunks("not json").is_empty());
    }

    #[test]
    fn test_parse_chunks_missing_fields_use_defaults() {
        let raw = r#"{"chunks":[{}]}"#;
        let chunks = parse_chunks(raw);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].id, "");
        assert_eq!(chunks[0].content, "");
        assert_eq!(chunks[0].position, 0);
        assert_eq!(chunks[0].priority, "medium");
    }

    #[test]
    fn test_parse_graph_nodes_basic() {
        let raw = r#"{"nodes":[{"id":"n1","name":"Test","type":"entity","weight":0.9}]}"#;
        let nodes = parse_graph_nodes(raw);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "Test");
        assert_eq!(nodes[0].node_type, "entity");
    }

    #[test]
    fn test_parse_graph_stats_present() {
        let raw = r#"{"stats":{"count":42}}"#;
        let stats = parse_graph_stats(raw);
        assert!(stats.is_some());
    }

    #[test]
    fn test_parse_graph_stats_absent() {
        assert!(parse_graph_stats(r#"{"other":1}"#).is_none());
    }
}
