use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::compress::{engine, prompt, token_estimate};
use crate::services::llm::stream;
use tokio_util::sync::CancellationToken;

pub async fn try_auto_compress(
    on_event: &AgentEventEmitter,
    provider_id: &str,
    model: &str,
    messages: &mut Vec<ChatMessage>,
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

    let last_assistant = messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant")
        .cloned();

    let compress_msgs = engine::build_compression_request_content(messages, None);
    match stream::collect_chat_silent(provider_id, model, &compress_msgs, cancel.clone()).await {
        Ok(result) => {
            let summary = prompt::extract_summary(&result.content);
            engine::apply_compression(messages, &summary, true);
            if let Some(last) = &last_assistant {
                messages.push(last.clone());
            }
            save_compressed_session(session_id, &summary, last_assistant.as_ref()).await;
        }
        Err(e) => {
            if !cancel.is_cancelled() {
                eprintln!("[compress] Échec compression LLM : {e}");
            }
        }
    }

    let _ = on_event.send(StreamEvent::Compressing { status: "done".to_string() });
    let _ = on_event.send(StreamEvent::CompressionComplete {});
}

async fn save_compressed_session(
    session_id: &str,
    summary: &str,
    last_assistant: Option<&ChatMessage>,
) {
    let summary_content = prompt::format_summary_message(summary, true);
    let summary_chat = ChatMessage {
        role: "assistant".to_string(),
        content: summary_content.clone(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None, reasoning_content: None,
    };
    let summary_tokens = token_estimate::estimate_tokens(&[summary_chat]) as u32;

    let summary_msg = AgentMessage {
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

    let mut session_messages = vec![summary_msg];

    if let Some(last) = last_assistant {
        let last_tokens = token_estimate::estimate_tokens(&[last.clone()]) as u32;
        session_messages.push(AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: last.content.clone(),
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_activities: None,
            segments: None,
            files: vec![],
            timestamp: chrono::Utc::now(),
            tokens: last_tokens,
            skill_names: None,
        });
    }

    if let Ok(mut session) = session_store::get(session_id).await {
        session.messages = session_messages;
        session.accumulated_tokens = session.messages.iter().map(|m| m.tokens).sum();
        let _ = session_store::save(&session).await;
    }
}
