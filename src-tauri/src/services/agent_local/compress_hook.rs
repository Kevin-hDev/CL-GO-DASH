use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::compress::{engine, prompt, token_estimate};
use tokio_util::sync::CancellationToken;

pub async fn try_auto_compress(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    model: &str,
    session_id: &str,
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
        token_estimate::estimate_tokens(messages)
    };
    if !engine::should_auto_compress(
        config.compression_enabled,
        native_context,
        configured_context,
        used,
        config.compression_threshold,
    ) {
        return;
    }

    let _ = on_event.send(StreamEvent::Compressing { status: "start".to_string() });

    let compress_msgs = engine::build_compression_request_content(messages, None);
    match ollama_stream::collect_chat(model, compress_msgs).await {
        Ok((content, _)) => {
            let summary = prompt::extract_summary(&content);
            engine::apply_compression(messages, &summary, true);
            save_compressed_session(session_id, &summary).await;
        }
        Err(e) => {
            if !cancel.is_cancelled() {
                eprintln!("[compress] Échec compression Ollama : {e}");
            }
        }
    }

    let _ = on_event.send(StreamEvent::Compressing { status: "done".to_string() });
    let _ = on_event.send(StreamEvent::CompressionComplete {});
}

async fn save_compressed_session(session_id: &str, summary: &str) {
    let summary_content = prompt::format_summary_message(summary, true);
    let summary_chat = ChatMessage {
        role: "assistant".to_string(),
        content: summary_content.clone(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
    };
    let summary_tokens = token_estimate::estimate_tokens(&[summary_chat]) as u32;

    let compressed_msg = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: summary_content,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: summary_tokens,
        skill_names: None,
    };

    if let Ok(mut session) = session_store::get(session_id).await {
        session.messages = vec![compressed_msg];
        session.accumulated_tokens = summary_tokens;
        let _ = session_store::save(&session).await;
    }
}
