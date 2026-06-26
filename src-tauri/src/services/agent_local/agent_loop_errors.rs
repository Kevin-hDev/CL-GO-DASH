use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;

pub async fn send(on_event: &AgentEventEmitter, session_id: &str, request_id: &str, message: &str) {
    let diagnostic =
        super::stream_diagnostics::record_failure(session_id, Some(request_id), message, false)
            .await;
    let _ = on_event.send(StreamEvent::Error {
        message: message.to_string(),
        is_connection: false,
        diagnostic,
    });
}

pub async fn max_turns(on_event: &AgentEventEmitter, session_id: &str, request_id: &str) {
    send(on_event, session_id, request_id, "Limite de tours atteinte").await;
}
