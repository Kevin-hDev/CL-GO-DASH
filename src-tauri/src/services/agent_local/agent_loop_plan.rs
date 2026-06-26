use super::plan_mode_controller::{self, PlanModeDecision};
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, StreamResult};

pub enum PlanLoopAction {
    Accept,
    Retry,
    Stop(&'static str),
}

pub async fn active(session_id: &str, fallback: bool) -> bool {
    fallback && super::tool_plan::is_enabled(session_id).await
}

pub async fn check_result(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    session_id: &str,
    request_id: &str,
    result: &StreamResult,
    active: bool,
    repair_count: usize,
) -> PlanLoopAction {
    if !active {
        return PlanLoopAction::Accept;
    }
    match plan_mode_controller::evaluate(session_id, result, repair_count).await {
        PlanModeDecision::Accept => {
            if result.tool_calls.is_empty() {
                super::stream_buffer::emit_buffered_content(on_event, result);
            }
            PlanLoopAction::Accept
        }
        PlanModeDecision::Retry(correction) => {
            messages.push(plan_mode_controller::correction_message(correction));
            PlanLoopAction::Retry
        }
        PlanModeDecision::Fail(message) => {
            super::plan_mode_debug::workflow_failed(session_id, request_id, message);
            PlanLoopAction::Stop(message)
        }
    }
}
