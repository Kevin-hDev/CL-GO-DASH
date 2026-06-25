use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::stream_events::AgentEventEmitter;
use super::types_interactive::{AgentInteractiveAnswer, AgentInteractiveQuestion};
use super::types_ollama::StreamEvent;
use super::types_tools::ToolResult;

pub async fn execute(
    args: &serde_json::Value,
    on_event: &AgentEventEmitter,
    cancel: CancellationToken,
) -> ToolResult {
    let questions = match super::tool_interactive_parse::parse_questions(args) {
        Ok(questions) => questions,
        Err(err) => return ToolResult::err(err),
    };
    match request(on_event, questions, cancel).await {
        Ok(answers) => ToolResult::ok(result_json(&answers)),
        Err(err) => ToolResult::err(err),
    }
}

pub async fn respond(id: String, answers: Vec<AgentInteractiveAnswer>) -> Result<(), String> {
    super::interactive_choice_gate::respond(&id, answers).await
}

async fn request(
    on_event: &AgentEventEmitter,
    questions: Vec<AgentInteractiveQuestion>,
    cancel: CancellationToken,
) -> Result<Vec<AgentInteractiveAnswer>, String> {
    super::interactive_choice_gate::request(on_event, questions, cancel).await
}

fn result_json(answers: &[AgentInteractiveAnswer]) -> String {
    let selected_labels: Vec<_> = answers
        .iter()
        .map(|answer| answer.selected_labels.clone())
        .collect();
    let custom_answers: Vec<_> = answers
        .iter()
        .filter_map(|answer| answer.custom_answer.clone())
        .collect();
    json!({
        "completed": true,
        "answers": answers,
        "selected_labels": selected_labels,
        "custom_answers": custom_answers,
    })
    .to_string()
}

pub(crate) fn emit_request(
    on_event: &AgentEventEmitter,
    id: String,
    questions: Vec<AgentInteractiveQuestion>,
) {
    let total = questions.len();
    let _ = on_event.send(StreamEvent::InteractiveChoiceRequest {
        id,
        questions,
        current_index: 0,
        total,
    });
}
