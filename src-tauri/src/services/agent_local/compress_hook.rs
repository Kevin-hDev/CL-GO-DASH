//! Hook de compression automatique pour la boucle Ollama.
//!
//! Déclenche la compression quand le seuil de tokens est atteint.
//! Utilise `collect_chat` (non-streaming) pour ne pas polluer le frontend.

use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use tokio_util::sync::CancellationToken;

pub async fn try_auto_compress(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    native_context: u64,
    configured_context: u64,
    last_context_tokens: u32,
    cancel: CancellationToken,
) {
    let config = match crate::services::config::read_config() {
        Ok(c) => c.advanced,
        Err(_) => return,
    };
    let used = if last_context_tokens > 0 {
        last_context_tokens as usize
    } else {
        crate::services::compress::token_estimate::estimate_tokens(messages)
    };
    if !crate::services::compress::engine::should_auto_compress(
        config.compression_enabled,
        native_context,
        configured_context,
        used,
        config.compression_threshold,
    ) {
        return;
    }

    let _ = on_event.send(StreamEvent::Compressing { status: "start".to_string() });

    let compress_msgs =
        crate::services::compress::engine::build_compression_request_content(messages, None);
    match ollama_stream::collect_chat(model, compress_msgs).await {
        Ok((content, _)) => {
            let summary = crate::services::compress::prompt::extract_summary(&content);
            crate::services::compress::engine::apply_compression(messages, &summary, true);
        }
        Err(e) => {
            if !cancel.is_cancelled() {
                eprintln!("[compress] Échec compression Ollama : {e}");
            }
        }
    }

    let _ = on_event.send(StreamEvent::Compressing { status: "done".to_string() });
}
