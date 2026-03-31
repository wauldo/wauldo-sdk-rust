//! SSE streaming chat completion example

use wauldo::{ChatMessage, ChatRequest, HttpClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::localhost()?;

    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("Explain async/await in Rust in 3 bullet points."),
    ];
    let req = ChatRequest::new("qwen2.5:7b", messages);

    // Stream response token by token
    let mut rx = client.chat_stream(req).await?;
    print!("Assistant: ");
    while let Some(chunk) = rx.recv().await {
        match chunk {
            Ok(text) => print!("{}", text),
            Err(e) => {
                eprintln!("\nStream error: {}", e);
                break;
            }
        }
    }
    println!();

    Ok(())
}
