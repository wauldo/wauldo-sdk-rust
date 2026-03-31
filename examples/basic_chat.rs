//! Basic chat completion example using the Wauldo SDK

use wauldo::{ChatMessage, ChatRequest, HttpClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::localhost()?;

    // List available models
    let models = client.list_models().await?;
    println!("Available models:");
    for m in &models.data {
        println!("  - {}", m.id);
    }

    // Pick first model or fallback
    let model = models
        .data
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "qwen2.5:7b".to_string());

    // Send a chat request
    let messages = vec![
        ChatMessage::system("You are a helpful assistant. Be concise."),
        ChatMessage::user("What is Rust's ownership model in one sentence?"),
    ];
    let req = ChatRequest::new(&model, messages);
    let resp = client.chat(req).await?;

    let answer = resp.choices[0]
        .message
        .content
        .as_deref()
        .unwrap_or("<no content>");
    println!("\nAssistant: {}", answer);
    println!("Tokens used: {}", resp.usage.total_tokens);

    Ok(())
}
