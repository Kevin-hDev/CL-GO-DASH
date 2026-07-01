use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult, TokenPhase};
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
        emit_token(on_event, content, *token_count, *first_token, None);
    }
}

pub fn emit_buffered_content(on_event: &AgentEventEmitter, result: &StreamResult, phase: TokenPhase) {
    let mut token_count = 0;
    let first_token = Some(std::time::Instant::now());
    for chunk in &result.content_chunks {
        token_count += 1;
        emit_token(on_event, chunk.clone(), token_count, first_token, Some(phase.clone()));
    }
}

pub fn finalize_content_phase(
    on_event: &AgentEventEmitter,
    result: &StreamResult,
    plan_active: bool,
) {
    if let Some(phase) = content_phase_for_result(result, plan_active) {
        if plan_active {
            emit_buffered_content(on_event, result, phase);
        } else {
            let _ = on_event.send(StreamEvent::ContentPhase { phase });
        }
    }
}

pub fn content_phase_for_result(result: &StreamResult, plan_active: bool) -> Option<TokenPhase> {
    if result.content_chunks.is_empty() {
        return None;
    }
    if plan_active && !result.tool_calls.is_empty() {
        return None;
    }
    Some(if result.tool_calls.is_empty() {
        TokenPhase::Final
    } else {
        TokenPhase::Work
    })
}

fn emit_token(
    on_event: &AgentEventEmitter,
    content: String,
    token_count: u32,
    first_token: Option<std::time::Instant>,
    phase: Option<TokenPhase>,
) {
    let tps = compute_tps(token_count, first_token);
    let _ = on_event.send(StreamEvent::Token {
        content,
        token_count,
        tps,
        phase,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_plain_turn_as_final() {
        let result = StreamResult {
            content_chunks: vec!["done".into()],
            ..Default::default()
        };

        assert!(matches!(
            content_phase_for_result(&result, false),
            Some(TokenPhase::Final)
        ));
    }

    #[test]
    fn classifies_tool_turn_as_work() {
        let result = StreamResult {
            content_chunks: vec!["working".into()],
            tool_calls: vec![("bash".into(), serde_json::json!({}))],
            ..Default::default()
        };

        assert!(matches!(
            content_phase_for_result(&result, false),
            Some(TokenPhase::Work)
        ));
    }

    #[test]
    fn hides_plan_mode_tool_content() {
        let result = StreamResult {
            content_chunks: vec!["hidden".into()],
            tool_calls: vec![("write_plan".into(), serde_json::json!({}))],
            ..Default::default()
        };

        assert!(content_phase_for_result(&result, true).is_none());
    }

    #[test]
    fn token_phase_serializes_when_present() {
        let event = StreamEvent::Token {
            content: "answer".into(),
            token_count: 1,
            tps: 0.0,
            phase: Some(TokenPhase::Final),
        };

        let value = serde_json::to_value(event).expect("serialize token");
        assert_eq!(value["data"]["phase"], "final");
    }

    #[test]
    fn content_phase_serializes_when_present() {
        let event = StreamEvent::ContentPhase {
            phase: TokenPhase::Work,
        };

        let value = serde_json::to_value(event).expect("serialize content phase");
        assert_eq!(value["event"], "contentPhase");
        assert_eq!(value["data"]["phase"], "work");
    }
}
