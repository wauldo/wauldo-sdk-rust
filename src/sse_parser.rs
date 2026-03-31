//! SSE stream parser for chat completion streaming

use crate::error::{Error, Result};
use crate::http_types::ChatChunk;
use tokio::sync::mpsc;

/// Parse SSE stream and send content chunks through channel
pub async fn parse_sse_stream(
    resp: reqwest::Response,
    tx: mpsc::Sender<Result<String>>,
) -> Result<()> {
    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buf = Vec::<u8>::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buf.extend_from_slice(&bytes);
                if process_byte_buffer(&mut buf, &tx).await? {
                    return Ok(());
                }
            }
            Err(e) => {
                // Best-effort send; receiver may already be dropped
                let _ = tx.send(Err(Error::connection(e.to_string()))).await;
                return Err(Error::connection(e.to_string()));
            }
        }
    }
    Ok(())
}

/// Process buffered bytes, decoding complete lines and extracting SSE deltas.
/// Returns `true` when [DONE] is received.
async fn process_byte_buffer(buf: &mut Vec<u8>, tx: &mpsc::Sender<Result<String>>) -> Result<bool> {
    while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
        let line_bytes = buf[..pos].to_vec();
        buf.drain(..=pos);
        let line = match String::from_utf8(line_bytes) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Non-UTF8 SSE line skipped: {}", e);
                continue;
            }
        };
        let trimmed = line.trim();
        if let Some(data) = trimmed.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Ok(true);
            }
            match serde_json::from_str::<ChatChunk>(data) {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            if tx.send(Ok(content.clone())).await.is_err() {
                                return Ok(true);
                            }
                        }
                    }
                }
                Err(_) => {
                    tracing::warn!("Malformed SSE chunk skipped: {}", data);
                }
            }
        }
    }
    Ok(false)
}
