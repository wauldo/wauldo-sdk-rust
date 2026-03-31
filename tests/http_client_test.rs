//! Tests for Wauldo HTTP Client (REST API)

use wauldo::{
    ChatMessage, ChatRequest, ChatResponse, Chunk, EmbeddingInput, HttpClient, HttpConfig,
    MockHttpClient, Model, ModelList, RagQueryResponse, RagUploadResponse, RetrievalResult,
};

// ============================================================================
// Client Construction
// ============================================================================

#[test]
fn test_http_client_localhost() {
    let client = HttpClient::localhost();
    assert!(client.is_ok());
}

#[test]
fn test_http_client_with_url() {
    let client = HttpClient::with_url("http://example.com:8080");
    assert!(client.is_ok());
}

#[test]
fn test_http_client_with_api_key() {
    let config = HttpConfig {
        base_url: "http://localhost:3000".to_string(),
        api_key: Some("sk-test-key".to_string()),
        timeout_secs: 30,
        ..Default::default()
    };
    let client = HttpClient::new(config);
    assert!(client.is_ok());
}

#[test]
fn test_http_config_default() {
    let config = HttpConfig::default();
    assert_eq!(config.base_url, "http://localhost:3000");
    assert!(config.api_key.is_none());
    assert_eq!(config.timeout_secs, 120);
}

// ============================================================================
// Type Builders
// ============================================================================

#[test]
fn test_chat_message_user() {
    let msg = ChatMessage::user("Hello");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content.as_deref(), Some("Hello"));
    assert!(msg.name.is_none());
}

#[test]
fn test_chat_message_system() {
    let msg = ChatMessage::system("You are helpful");
    assert_eq!(msg.role, "system");
    assert_eq!(msg.content.as_deref(), Some("You are helpful"));
}

#[test]
fn test_chat_message_assistant() {
    let msg = ChatMessage::assistant("Sure!");
    assert_eq!(msg.role, "assistant");
    assert_eq!(msg.content.as_deref(), Some("Sure!"));
}

#[test]
fn test_chat_request_new() {
    let req = ChatRequest::new("gpt-4", vec![ChatMessage::user("Hi")]);
    assert_eq!(req.model, "gpt-4");
    assert_eq!(req.messages.len(), 1);
    assert!(req.temperature.is_none());
    assert!(req.max_tokens.is_none());
    assert!(req.stream.is_none());
}

#[test]
fn test_chat_request_serialization() {
    let req = ChatRequest::new(
        "qwen2.5:7b",
        vec![
            ChatMessage::system("Be concise"),
            ChatMessage::user("What is Rust?"),
        ],
    );
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"model\":\"qwen2.5:7b\""));
    assert!(json.contains("\"role\":\"system\""));
    assert!(json.contains("\"role\":\"user\""));
    // Optional fields should be absent (skip_serializing_if)
    assert!(!json.contains("\"temperature\""));
    assert!(!json.contains("\"stream\""));
}

// ============================================================================
// Response Deserialization
// ============================================================================

#[test]
fn test_chat_response_deserialization() {
    let json = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1709900000,
        "model": "qwen2.5:7b",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "Hello!"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
    }"#;
    let resp: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.id, "chatcmpl-123");
    assert_eq!(resp.model, "qwen2.5:7b");
    assert_eq!(resp.choices.len(), 1);
    assert_eq!(resp.choices[0].message.content.as_deref(), Some("Hello!"));
    assert_eq!(resp.choices[0].finish_reason.as_deref(), Some("stop"));
    assert_eq!(resp.usage.prompt_tokens, 10);
    assert_eq!(resp.usage.total_tokens, 15);
}

#[test]
fn test_model_list_deserialization() {
    let json = r#"{
        "object": "list",
        "data": [
            {"id": "qwen2.5:7b", "object": "model", "created": 1709000000, "owned_by": "ollama"},
            {"id": "llama3:8b", "object": "model", "created": 1709000001, "owned_by": "ollama"}
        ]
    }"#;
    let list: ModelList = serde_json::from_str(json).unwrap();
    assert_eq!(list.data.len(), 2);
    assert_eq!(list.data[0].id, "qwen2.5:7b");
    assert_eq!(list.data[1].owned_by, "ollama");
}

#[test]
fn test_embedding_input_single_serialization() {
    let input = EmbeddingInput::Single("hello world".to_string());
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, "\"hello world\"");
}

#[test]
fn test_embedding_input_multiple_serialization() {
    let input = EmbeddingInput::Multiple(vec!["a".into(), "b".into()]);
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, "[\"a\",\"b\"]");
}

#[test]
fn test_embedding_response_deserialization() {
    let json = r#"{
        "data": [{"embedding": [0.1, 0.2, 0.3], "index": 0}],
        "model": "bge-small-en",
        "usage": {"prompt_tokens": 5, "total_tokens": 5}
    }"#;
    let resp: wauldo::EmbeddingResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].embedding.len(), 3);
    assert_eq!(resp.model, "bge-small-en");
}

#[test]
fn test_rag_upload_request_serialization() {
    let req = wauldo::RagUploadRequest {
        content: "My document".into(),
        filename: Some("doc.txt".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"content\":\"My document\""));
    assert!(json.contains("\"filename\":\"doc.txt\""));
}

#[test]
fn test_rag_query_response_deserialization() {
    let json = r#"{
        "answer": "Rust is a systems language",
        "sources": [
            {"document_id": "doc-1", "content": "Rust...", "score": 0.95}
        ]
    }"#;
    let resp: wauldo::RagQueryResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.answer, "Rust is a systems language");
    assert_eq!(resp.sources.len(), 1);
    assert!((resp.sources[0].score - 0.95).abs() < 0.001);
}

#[test]
fn test_orchestrator_request_serialization() {
    let req = wauldo::OrchestratorRequest {
        prompt: "Analyze this code".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"prompt\":\"Analyze this code\""));
}

#[test]
fn test_orchestrator_response_deserialization() {
    let json = r#"{"final_output": "The code looks good"}"#;
    let resp: wauldo::OrchestratorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.final_output, "The code looks good");
}

// ============================================================================
// SSE Chunk Parsing
// ============================================================================

#[test]
fn test_chat_chunk_deserialization() {
    let json = r#"{
        "id": "chatcmpl-stream-1",
        "choices": [{
            "delta": {"content": "Hello"},
            "finish_reason": null
        }]
    }"#;
    let chunk: wauldo::ChatChunk = serde_json::from_str(json).unwrap();
    assert_eq!(chunk.id, "chatcmpl-stream-1");
    assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hello"));
    assert!(chunk.choices[0].finish_reason.is_none());
}

#[test]
fn test_chat_chunk_finish() {
    let json = r#"{
        "id": "chatcmpl-stream-1",
        "choices": [{
            "delta": {},
            "finish_reason": "stop"
        }]
    }"#;
    let chunk: wauldo::ChatChunk = serde_json::from_str(json).unwrap();
    assert!(chunk.choices[0].delta.content.is_none());
    assert_eq!(chunk.choices[0].finish_reason.as_deref(), Some("stop"));
}

// ============================================================================
// MockHttpClient Tests
// ============================================================================

/// Helper: build a realistic ChatResponse for mock tests
fn make_chat_response(content: &str) -> ChatResponse {
    serde_json::from_value(serde_json::json!({
        "id": "chatcmpl-mock-1",
        "object": "chat.completion",
        "created": 1709900000_i64,
        "model": "mock-model",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": content},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 8, "completion_tokens": 4, "total_tokens": 12}
    }))
    .expect("valid ChatResponse JSON")
}

#[tokio::test]
async fn test_mock_chat_returns_configured_response() {
    let expected = make_chat_response("Rust is a systems language.");
    let mock = MockHttpClient::new().with_chat_response(expected);

    let req = ChatRequest::new(
        "any-model",
        vec![
            ChatMessage::system("Be concise"),
            ChatMessage::user("What is Rust?"),
        ],
    );
    let resp = mock.chat(req).await.unwrap();

    assert_eq!(resp.id, "chatcmpl-mock-1");
    assert_eq!(resp.model, "mock-model");
    assert_eq!(resp.choices.len(), 1);
    assert_eq!(
        resp.choices[0].message.content.as_deref(),
        Some("Rust is a systems language.")
    );
    assert_eq!(resp.usage.total_tokens, 12);
}

#[tokio::test]
async fn test_mock_chat_simple() {
    let expected = make_chat_response("Hello there!");
    let mock = MockHttpClient::new().with_chat_response(expected);

    let req = ChatRequest::quick("gpt-4", "Hi");
    let resp = mock.chat(req).await.unwrap();

    assert_eq!(resp.text(), Some("Hello there!"));
}

#[tokio::test]
async fn test_mock_list_models() {
    let models = vec![
        serde_json::from_value::<Model>(serde_json::json!({
            "id": "qwen2.5:7b",
            "object": "model",
            "created": 1709000000_i64,
            "owned_by": "ollama"
        }))
        .unwrap(),
        serde_json::from_value::<Model>(serde_json::json!({
            "id": "llama3:8b",
            "object": "model",
            "created": 1709000001_i64,
            "owned_by": "ollama"
        }))
        .unwrap(),
    ];
    let mock = MockHttpClient::new().with_models(models);

    let list = mock.list_models().await.unwrap();
    assert_eq!(list.object, "list");
    assert_eq!(list.data.len(), 2);
    assert_eq!(list.data[0].id, "qwen2.5:7b");
    assert_eq!(list.data[1].id, "llama3:8b");
}

#[tokio::test]
async fn test_mock_rag_upload_and_query() {
    let upload_resp: RagUploadResponse = serde_json::from_value(serde_json::json!({
        "document_id": "doc-42",
        "chunks_count": 5
    }))
    .unwrap();
    let query_resp: RagQueryResponse = serde_json::from_value(serde_json::json!({
        "answer": "The answer is 42",
        "sources": [
            {"document_id": "doc-42", "content": "chunk text", "score": 0.98}
        ]
    }))
    .unwrap();

    let mock = MockHttpClient::new()
        .with_rag_upload(upload_resp)
        .with_rag_query(query_resp);

    let upload = mock
        .rag_upload("some document text", Some("test.txt".into()))
        .await
        .unwrap();
    assert_eq!(upload.document_id, "doc-42");
    assert_eq!(upload.chunks_count, 5);

    let query = mock
        .rag_query("What is the answer?", Some(3))
        .await
        .unwrap();
    assert_eq!(query.answer, "The answer is 42");
    assert_eq!(query.sources.len(), 1);
    assert!((query.sources[0].score - 0.98).abs() < 0.001);
}

#[tokio::test]
async fn test_mock_unconfigured_returns_error() {
    let mock = MockHttpClient::new();

    let err = mock.list_models().await;
    assert!(err.is_err());

    let err = mock.chat(ChatRequest::quick("m", "hi")).await;
    assert!(err.is_err());

    let err = mock.rag_upload("text", None).await;
    assert!(err.is_err());

    let err = mock.rag_query("q", None).await;
    assert!(err.is_err());
}

#[test]
fn test_chat_response_text_helper() {
    let resp = make_chat_response("Hello world");
    assert_eq!(resp.text(), Some("Hello world"));

    // Response with empty choices
    let empty: ChatResponse = serde_json::from_value(serde_json::json!({
        "id": "x", "object": "chat.completion", "created": 0_i64,
        "model": "m", "choices": [],
        "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
    }))
    .unwrap();
    assert_eq!(empty.text(), None);
}

#[test]
fn test_chat_request_quick_builder() {
    let req = ChatRequest::quick("gpt-4", "Explain Rust");
    assert_eq!(req.model, "gpt-4");
    assert_eq!(req.messages.len(), 1);
    assert_eq!(req.messages[0].role, "user");
    assert_eq!(req.messages[0].content.as_deref(), Some("Explain Rust"));
    assert!(req.temperature.is_none());
    assert!(req.max_tokens.is_none());
    assert!(req.stream.is_none());
}

#[test]
fn test_parse_chunks_from_json() {
    let json = r#"{
        "chunks": [
            {"id": "c1", "content": "First chunk", "position": 0, "priority": "high"},
            {"id": "c2", "content": "Second chunk", "position": 1, "priority": "medium"}
        ]
    }"#;
    let data: serde_json::Value = serde_json::from_str(json).unwrap();
    let items = data["chunks"].as_array().unwrap();
    let chunks: Vec<Chunk> = items
        .iter()
        .map(|c| serde_json::from_value(c.clone()).unwrap())
        .collect();

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].id, "c1");
    assert_eq!(chunks[0].content, "First chunk");
    assert_eq!(chunks[0].position, 0);
    assert_eq!(chunks[0].priority, "high");
    assert_eq!(chunks[1].id, "c2");
    assert_eq!(chunks[1].position, 1);
}

#[test]
fn test_parse_retrieval_results_from_json() {
    let json = r#"{
        "query": "What is Rust?",
        "results": [
            {"id": "r1", "content": "Rust is a systems language", "position": 0, "priority": "high"},
            {"id": "r2", "content": "It focuses on safety", "position": 1, "priority": "medium"}
        ],
        "raw_content": ""
    }"#;
    let result: RetrievalResult = serde_json::from_str(json).unwrap();

    assert_eq!(result.query, "What is Rust?");
    assert_eq!(result.results.len(), 2);
    assert_eq!(result.results[0].id, "r1");
    assert_eq!(result.results[0].content, "Rust is a systems language");
    assert_eq!(result.results[1].id, "r2");
    assert_eq!(result.results[1].priority, "medium");
}

#[test]
fn test_parse_chunks_malformed_json() {
    // Malformed JSON should not panic — serde returns Err
    let bad_json = r#"{ not valid json }"#;
    let result = serde_json::from_str::<serde_json::Value>(bad_json);
    assert!(result.is_err());

    // Valid JSON but missing expected fields — Chunk deser fails gracefully
    let missing_fields = r#"{"unexpected": true}"#;
    let result = serde_json::from_str::<Chunk>(missing_fields);
    assert!(result.is_err());

    // Empty chunks array parses fine
    let empty = r#"{"chunks": []}"#;
    let data: serde_json::Value = serde_json::from_str(empty).unwrap();
    let items = data["chunks"].as_array().unwrap();
    assert!(items.is_empty());
}
