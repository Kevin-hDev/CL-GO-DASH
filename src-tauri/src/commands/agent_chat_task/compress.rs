use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamEvent};
use tokio_util::sync::CancellationToken;

pub(crate) fn is_compress_command(messages: &[ChatMessage]) -> bool {
    messages
        .last()
        .map(|m| m.role == "user" && m.content.trim() == "/compress")
        .unwrap_or(false)
}

pub(crate) async fn handle_compress_command(
    on_event: &AgentEventEmitter,
    session_id: &str,
    messages: &[ChatMessage],
    model: &str,
    provider: &str,
    cancel: CancellationToken,
) -> Result<(), String> {
    use crate::services::agent_local::session_store;
    use crate::services::agent_local::types_session::AgentMessage;
    use crate::services::compress::{engine, prompt};

    let _ = on_event.send(StreamEvent::Compressing {
        status: "start".to_string(),
    });

    let msgs_without_command: Vec<ChatMessage> = messages
        .iter()
        .filter(|m| !(m.role == "user" && m.content.trim() == "/compress"))
        .cloned()
        .collect();

    let compress_msgs = engine::build_compression_request_content(&msgs_without_command, None);
    let summary_raw = collect_summary(provider, model, compress_msgs, cancel).await?;
    let summary = prompt::extract_summary(&summary_raw);
    let summary_content = prompt::format_summary_message(&summary, false);
    let summary_tokens = estimate_summary_tokens(&summary_content);

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
        tokens: summary_tokens as u32,
        skill_names: None,
    };

    if let Ok(mut session) = session_store::get(session_id).await {
        session.messages = vec![compressed_msg];
        session.accumulated_tokens = summary_tokens as u32;
        let _ = session_store::save(&session).await;
    }

    send_compression_done(on_event, summary_tokens as u32);
    Ok(())
}

async fn collect_summary(
    provider: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    cancel: CancellationToken,
) -> Result<String, String> {
    if provider == "ollama" {
        return crate::services::agent_local::ollama_stream::collect_chat(model, messages)
            .await
            .map(|(content, _)| content)
            .map_err(|err| format!("Compression Ollama : {err}"));
    }
    crate::services::llm::stream::collect_chat_silent(provider, model, &messages, cancel)
        .await
        .map(|result| result.content)
        .map_err(|err| format!("Compression LLM : {err}"))
}

fn estimate_summary_tokens(summary_content: &str) -> usize {
    let summary_chat_msg = ChatMessage {
        role: "assistant".to_string(),
        content: summary_content.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    };
    crate::services::compress::token_estimate::estimate_tokens(&[summary_chat_msg])
}

fn send_compression_done(on_event: &AgentEventEmitter, summary_tokens: u32) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
    let _ = on_event.send(StreamEvent::CompressionComplete {});
    let _ = on_event.send(StreamEvent::Done {
        eval_count: 0,
        eval_duration_ns: 0,
        final_tps: 0.0,
        prompt_tokens: 0,
        context_tokens: summary_tokens,
    });
}
