use crate::services::agent_local::ollama_stream;
use crate::services::agent_local::session_store;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use crate::services::agent_local::types_session::AgentMessage;
use crate::services::compress::{context_capsules, engine, prompt, summary_budget, token_estimate};
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
    let estimated = token_estimate::estimate_tokens(messages);
    let used = context_used_for_compression(last_context_tokens, estimated);
    if !engine::should_auto_compress(
        config.compression_enabled,
        native_context,
        configured_context,
        used,
        config.compression_threshold,
    ) {
        return;
    }

    let _ = on_event.send(StreamEvent::Compressing {
        status: "start".to_string(),
    });

    // Garder la dernière réponse assistant pour ne pas la perdre
    let last_assistant = messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant")
        .cloned();

    let file_context = context_capsules::recent_file_context_message(messages, configured_context);
    let summary_instruction = summary_budget::summary_instruction(configured_context);
    let compress_msgs =
        engine::build_compression_request_content(messages, summary_instruction.as_deref());
    match ollama_stream::collect_chat(model, compress_msgs).await {
        Ok((content, _)) => {
            let summary = prompt::extract_summary(&content);
            engine::apply_compression(messages, &summary, true);
            context_capsules::insert_after_system(messages, file_context.clone());
            // Rajouter la dernière réponse après compression
            if let Some(last) = &last_assistant {
                messages.push(last.clone());
            }
            save_compressed_session(
                session_id,
                &summary,
                file_context.as_ref(),
                last_assistant.as_ref(),
            )
            .await;
        }
        Err(e) => {
            if !cancel.is_cancelled() {
                eprintln!("[compress] Échec compression Ollama : {e}");
            }
        }
    }

    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
    let _ = on_event.send(StreamEvent::CompressionComplete {});
}

fn context_used_for_compression(last_context_tokens: u32, estimated_tokens: usize) -> usize {
    std::cmp::max(last_context_tokens as usize, estimated_tokens)
}

async fn save_compressed_session(
    session_id: &str,
    summary: &str,
    file_context: Option<&ChatMessage>,
    last_assistant: Option<&ChatMessage>,
) {
    let summary_content = prompt::format_summary_message(summary, true);
    let summary_chat = ChatMessage {
        role: "assistant".to_string(),
        content: summary_content.clone(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
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

    if let Some(file_context) = file_context {
        let context_tokens = token_estimate::estimate_tokens(std::slice::from_ref(file_context)) as u32;
        session_messages.push(AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: file_context.content.clone(),
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_activities: None,
            segments: None,
            files: vec![],
            timestamp: chrono::Utc::now(),
            tokens: context_tokens,
            skill_names: None,
        });
    }

    if let Some(last) = last_assistant {
        let last_tokens = token_estimate::estimate_tokens(std::slice::from_ref(last)) as u32;
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

#[cfg(test)]
mod tests {
    use super::context_used_for_compression;

    #[test]
    fn uses_estimate_when_current_messages_are_larger() {
        assert_eq!(context_used_for_compression(10_000, 12_000), 12_000);
    }

    #[test]
    fn uses_provider_count_when_it_is_larger() {
        assert_eq!(context_used_for_compression(15_000, 12_000), 15_000);
    }
}
