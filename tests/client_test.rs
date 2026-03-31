//! Tests for Wauldo Rust SDK

use wauldo::{
    AgentClient, ClientOptions, DetailLevel, Error, PlanOptions, ReasoningOptions, SourceType,
};

// ============================================================================
// Client Initialization Tests
// ============================================================================

#[test]
fn test_client_creation_default() {
    let _client = AgentClient::new();
}

#[test]
fn test_client_creation_with_options() {
    let _client = AgentClient::with_options(
        ClientOptions::new()
            .server_path("/custom/path")
            .timeout_ms(60000)
            .auto_connect(false),
    );
}

#[test]
fn test_client_options_builder() {
    let options = ClientOptions::new()
        .server_path("/path/to/server")
        .timeout_ms(45000)
        .auto_connect(true);

    assert_eq!(options.server_path, Some("/path/to/server".to_string()));
    assert_eq!(options.timeout_ms, 45000);
    assert!(options.auto_connect);
}

#[test]
fn test_client_options_default() {
    let options = ClientOptions::default();
    assert!(options.server_path.is_none());
    assert_eq!(options.timeout_ms, 30000);
    assert!(options.auto_connect);
}

// ============================================================================
// Reasoning Options Tests
// ============================================================================

#[test]
fn test_reasoning_options_builder() {
    let options = ReasoningOptions::new().depth(5).branches(4);

    assert_eq!(options.depth, Some(5));
    assert_eq!(options.branches, Some(4));
}

#[test]
fn test_reasoning_options_default() {
    let options = ReasoningOptions::default();
    assert!(options.depth.is_none());
    assert!(options.branches.is_none());
}

// ============================================================================
// Plan Options Tests
// ============================================================================

#[test]
fn test_plan_options_builder() {
    let options = PlanOptions::new()
        .context("Using JWT tokens")
        .max_steps(8)
        .detail_level(DetailLevel::Detailed);

    assert_eq!(options.context, Some("Using JWT tokens".to_string()));
    assert_eq!(options.max_steps, Some(8));
    assert!(matches!(options.detail_level, Some(DetailLevel::Detailed)));
}

#[test]
fn test_plan_options_default() {
    let options = PlanOptions::default();
    assert!(options.context.is_none());
    assert!(options.max_steps.is_none());
    assert!(options.detail_level.is_none());
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_reason_empty_problem() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.reason("").await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("problem".to_string()));
    } else {
        panic!("Expected validation error");
    }
}

#[tokio::test]
async fn test_reason_whitespace_problem() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.reason("   ").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reason_invalid_depth() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client
        .reason_with_options("test", ReasoningOptions::new().depth(0))
        .await;
    assert!(result.is_err());

    let result = client
        .reason_with_options("test", ReasoningOptions::new().depth(11))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reason_invalid_branches() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client
        .reason_with_options("test", ReasoningOptions::new().branches(0))
        .await;
    assert!(result.is_err());

    let result = client
        .reason_with_options("test", ReasoningOptions::new().branches(11))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_extract_concepts_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.extract_concepts("", SourceType::Text).await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("text".to_string()));
    }
}

#[tokio::test]
async fn test_chunk_document_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.chunk_document("", 512).await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("content".to_string()));
    }
}

#[tokio::test]
async fn test_retrieve_context_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.retrieve_context("", 5).await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("query".to_string()));
    }
}

#[tokio::test]
async fn test_summarize_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.summarize("").await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("content".to_string()));
    }
}

#[tokio::test]
async fn test_search_knowledge_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.search_knowledge("", 10).await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("query".to_string()));
    }
}

#[tokio::test]
async fn test_add_to_knowledge_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.add_to_knowledge("").await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("text".to_string()));
    }
}

#[tokio::test]
async fn test_plan_task_empty() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client.plan_task("").await;
    assert!(result.is_err());

    if let Err(Error::Validation { field, .. }) = result {
        assert_eq!(field, Some("task".to_string()));
    }
}

#[tokio::test]
async fn test_plan_task_invalid_max_steps() {
    let mut client = AgentClient::with_options(ClientOptions::new().auto_connect(false));

    let result = client
        .plan_task_with_options("test", PlanOptions::new().max_steps(0))
        .await;
    assert!(result.is_err());

    let result = client
        .plan_task_with_options("test", PlanOptions::new().max_steps(21))
        .await;
    assert!(result.is_err());
}

// ============================================================================
// Error Tests
// ============================================================================

#[test]
fn test_error_validation() {
    let error = Error::validation("Test error");
    assert!(matches!(error, Error::Validation { .. }));

    let error = Error::validation_field("Test error", "field_name");
    if let Error::Validation { field, .. } = error {
        assert_eq!(field, Some("field_name".to_string()));
    }
}

#[test]
fn test_error_connection() {
    let error = Error::connection("Connection failed");
    assert!(matches!(error, Error::Connection(_)));
}

#[test]
fn test_error_server() {
    let error = Error::server(-32603, "Internal error");
    if let Error::Server { code, message, .. } = error {
        assert_eq!(code, -32603);
        assert_eq!(message, "Internal error");
    }
}

#[test]
fn test_error_display() {
    let error = Error::validation_field("Invalid input", "name");
    let display = format!("{}", error);
    assert!(display.contains("Invalid input"));
}

// ============================================================================
// Type Tests
// ============================================================================

#[test]
fn test_source_type() {
    let _text = SourceType::Text;
    let _code = SourceType::Code;
}

#[test]
fn test_detail_level_default() {
    let level = DetailLevel::default();
    assert!(matches!(level, DetailLevel::Normal));
}
