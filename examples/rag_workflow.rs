//! RAG workflow: upload a document then query it

use wauldo::{HttpClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::localhost()?;

    // Upload a document
    let content = "\
Rust is a systems programming language focused on safety, speed, and concurrency. \
It achieves memory safety without garbage collection through its ownership system. \
The borrow checker enforces rules at compile time, preventing data races and dangling pointers.";

    let upload = client
        .rag_upload(content, Some("rust_intro.txt".to_string()))
        .await?;
    println!(
        "Uploaded document '{}' ({} chunks)",
        upload.document_id, upload.chunks_count
    );

    // Query the RAG index
    let result = client
        .rag_query("How does Rust handle memory safety?", Some(3))
        .await?;
    println!("\nAnswer: {}", result.answer);
    println!("\nSources:");
    for src in &result.sources {
        println!("  - [score={:.2}] {}", src.score, src.content);
    }

    Ok(())
}
