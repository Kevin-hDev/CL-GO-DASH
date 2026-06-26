use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult};
use crate::services::stream_utils::compute_tps;

pub fn record_content(
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    content: String,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    buffer_content: bool,
) {
    result.content.push_str(&content);
    result.content_chunks.push(content.clone());
    *token_count += 1;
    if first_token.is_none() {
        *first_token = Some(std::time::Instant::now());
    }
    if !buffer_content {
        emit_token(on_event, content, *token_count, *first_token);
    }
}

pub fn emit_buffered_content(on_event: &AgentEventEmitter, result: &StreamResult) {
    let mut token_count = 0;
    let first_token = Some(std::time::Instant::now());
    for chunk in &result.content_chunks {
        token_count += 1;
        emit_token(on_event, chunk.clone(), token_count, first_token);
    }
}

fn emit_token(
    on_event: &AgentEventEmitter,
    content: String,
    token_count: u32,
    first_token: Option<std::time::Instant>,
) {
    let tps = compute_tps(token_count, first_token);
    let _ = on_event.send(StreamEvent::Token {
        content,
        token_count,
        tps,
    });
}
