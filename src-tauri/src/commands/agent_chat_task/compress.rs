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
    request_id: &str,
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
    crate::services::agent_local::stream_diagnostics::mark_phase(
        session_id,
        request_id,
        "compression",
        "Compression du contexte démarrée.",
    )
    .await;

    let msgs_without_command: Vec<ChatMessage> = messages
        .iter()
        .filter(|m| !(m.role == "user" && m.content.trim() == "/compress"))
        .cloned()
        .collect();

    let input_tokens =
        crate::services::compress::token_estimate::estimate_tokens(&msgs_without_command);
    let context = resolve_context_window(provider, model).await;
    let (summary_instruction, output_limit) =
        crate::services::compress::summary_budget::summary_instruction_for_input(
            context,
            input_tokens,
        );
    eprintln!(
        "[compress] manual start session={session_id} provider={provider} input_tokens={input_tokens} output_limit={output_limit}"
    );

    let compress_msgs = engine::build_compression_request_content(
        &msgs_without_command,
        summary_instruction.as_deref(),
    );
    let summary_raw =
        match collect_summary(provider, model, compress_msgs, output_limit, cancel).await {
            Ok(summary) => summary,
            Err(err) => {
                eprintln!("[compress] manual failed session={session_id}: {err}");
                send_compressing_done(on_event);
                return Err(err);
            }
        };
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
        session.accumulated_tokens =
            crate::services::token_counting::estimate_agent_messages_tokens(&session.messages);
        let _ = session_store::save(&session).await;
    }

    send_compression_done(on_event, summary_tokens as u32);
    eprintln!("[compress] manual done session={session_id} summary_tokens={summary_tokens}");
    Ok(())
}

async fn collect_summary(
    provider: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    output_limit: u32,
    cancel: CancellationToken,
) -> Result<String, String> {
    let timeout = crate::services::compress::timeouts::compression_timeout();
    if provider == "ollama" {
        let compression =
            crate::services::agent_local::ollama_stream::collect_chat_with_timeout_and_limit(
                model,
                messages,
                timeout,
                Some(output_limit),
            );
        return tokio::select! {
            _ = cancel.cancelled() => Err("Annulé".to_string()),
            result = compression => result
                .map(|(content, _)| content)
                .map_err(|err| format!("Compression Ollama : {err}")),
        };
    }
    crate::services::llm::stream::collect_chat_silent_for_compression(
        provider,
        model,
        &messages,
        output_limit,
        cancel,
    )
    .await
    .map(|result| result.content)
    .map_err(|err| format!("Compression LLM : {err}"))
}

async fn resolve_context_window(provider: &str, model: &str) -> u64 {
    let ctx = if provider == "ollama" {
        crate::services::compress::context_resolve::resolve_ollama(model).await
    } else {
        crate::services::compress::context_resolve::resolve_api(provider, model).await
    };
    ctx.configured
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
    send_compressing_done(on_event);
    let _ = on_event.send(StreamEvent::CompressionComplete {});
    let _ = on_event.send(StreamEvent::Done {
        eval_count: Some(0),
        eval_duration_ns: 0,
        final_tps: 0.0,
        prompt_tokens: Some(0),
        context_tokens: Some(summary_tokens),
    });
}

fn send_compressing_done(on_event: &AgentEventEmitter) {
    let _ = on_event.send(StreamEvent::Compressing {
        status: "done".to_string(),
    });
}
