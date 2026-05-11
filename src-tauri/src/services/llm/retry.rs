//! Logique de retry pour les appels LLM API.
//!
//! Gère les erreurs transitoires (429, 503, timeout) avec back-off progressif.

use super::stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent, StreamResult};
use tokio_util::sync::CancellationToken;

const MAX_RETRIES: usize = 5;
const RETRY_BASE_MS: u64 = 2000;

fn is_retryable_error(error: &str) -> bool {
    error.contains("429")
        || error.contains("rate limit")
        || error.contains("Rate limit")
        || error.contains("503")
        || error.contains("502")
        || error.contains("timeout")
        || error.contains("Timeout")
        || error.contains("ETIMEDOUT")
        || error.contains("ECONNRESET")
        || error.contains("Transport error")
        || error.contains("decoding response body")
        || error.contains("connection closed")
        || error.contains("SSE:")
}

pub async fn retry_stream(
    on_event: &AgentEventEmitter,
    provider_id: &str,
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let mut last_error = String::new();
    for attempt in 0..=MAX_RETRIES {
        if cancel.is_cancelled() {
            return Err("Annulé".to_string());
        }
        if attempt > 0 {
            let _ = on_event.send(StreamEvent::Error {
                message: format!("Retry {attempt}/{MAX_RETRIES} après erreur : {last_error}"),
                is_connection: false,
            });
            let delay = RETRY_BASE_MS * (1 << (attempt - 1));
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
        match stream::stream_chat_no_done(
            on_event,
            provider_id,
            model,
            messages,
            tools,
            think,
            cancel.clone(),
        )
        .await
        {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable_error(&e) && attempt < MAX_RETRIES => {
                last_error = e;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_error)
}
