use chrono::Utc;

use super::types_diagnostics::{
    AgentDiagnosticEvent, AgentDiagnosticRun, AgentDiagnosticTodo, AgentErrorDiagnosticSummary,
    AgentStreamFailure,
};
use super::types_session::AgentSession;
use super::types_todo::AgentTodoStatus;

pub(crate) const MAX_STREAM_FAILURES: usize = 20;
pub(crate) const MAX_DIAGNOSTIC_RUNS: usize = 20;
pub(crate) const MAX_DIAGNOSTIC_EVENTS: usize = 12;
const MAX_TEXT: usize = 200;

pub(crate) fn active_todo(session: &AgentSession) -> Option<AgentDiagnosticTodo> {
    let run = session
        .active_todo_run_id
        .as_ref()
        .and_then(|id| session.todo_runs.iter().find(|run| &run.id == id))?;
    let completed = run
        .todos
        .iter()
        .filter(|todo| todo.status == AgentTodoStatus::Completed)
        .count();
    let active_task = run.todos.iter().find_map(|todo| {
        (todo.status == AgentTodoStatus::InProgress)
            .then(|| todo.active_form.as_ref().unwrap_or(&todo.content).clone())
    });
    Some(AgentDiagnosticTodo {
        id: run.id.clone(),
        title: clip(&run.title),
        active_task: active_task.map(|value| clip(&value)),
        completed,
        total: run.todos.len(),
        progress: format!("{completed}/{}", run.todos.len()),
    })
}

pub(crate) fn push_failure(session: &mut AgentSession, message: &str, is_connection: bool) {
    let active = session
        .active_todo_run_id
        .as_ref()
        .and_then(|id| session.todo_runs.iter().find(|run| &run.id == id));
    session.stream_failures.push(AgentStreamFailure {
        code: safe_code(message),
        occurred_at: Utc::now(),
        is_connection,
        active_todo_run_id: active.map(|run| run.id.clone()),
        active_todo_title: active.map(|run| run.title.clone()),
    });
    trim(&mut session.stream_failures, MAX_STREAM_FAILURES);
}

pub(crate) fn apply_failure(
    session: &mut AgentSession,
    idx: usize,
    message: &str,
    is_connection: bool,
) {
    let todo = active_todo(session);
    let run = &mut session.diagnostic_runs[idx];
    let error_type = classify_error(message, is_connection);
    run.status = "failed".to_string();
    run.phase = "failed".to_string();
    run.severity = "error".to_string();
    run.error_type = Some(error_type.clone());
    run.ended_at = Some(Utc::now());
    run.updated_at = Utc::now();
    run.active_todo = todo;
    run.safe_summary = Some(safe_summary(run, &error_type));
    push_event(run, "failed", message, None, Some(&error_type));
}

pub(crate) fn summary_from_run(run: &AgentDiagnosticRun) -> AgentErrorDiagnosticSummary {
    AgentErrorDiagnosticSummary {
        request_id: run.request_id.clone(),
        phase: run.phase.clone(),
        error_type: run
            .error_type
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        last_tool_name: run.last_tool.as_ref().map(|tool| tool.name.clone()),
        safe_summary: run
            .safe_summary
            .clone()
            .unwrap_or_else(|| "Flux interrompu.".to_string()),
    }
}

pub(crate) fn push_event(
    run: &mut AgentDiagnosticRun,
    phase: &str,
    message: &str,
    tool_name: Option<&str>,
    error_type: Option<&str>,
) {
    run.updated_at = Utc::now();
    run.events
        .push(event(phase, message, tool_name, error_type));
    trim(&mut run.events, MAX_DIAGNOSTIC_EVENTS);
}

pub(crate) fn event(
    phase: &str,
    message: &str,
    tool_name: Option<&str>,
    error_type: Option<&str>,
) -> AgentDiagnosticEvent {
    AgentDiagnosticEvent {
        at: Utc::now(),
        phase: phase.to_string(),
        message: clip(message),
        tool_name: tool_name.map(clip),
        error_type: error_type.map(str::to_string),
    }
}

pub(crate) async fn update_run<F>(
    session_id: &str,
    request_id: &str,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(&mut AgentSession, &mut AgentDiagnosticRun),
{
    update_session(session_id, |session| {
        if let Some(idx) = find_run(session, request_id) {
            let mut run = session.diagnostic_runs.remove(idx);
            f(session, &mut run);
            session.diagnostic_runs.insert(idx, run);
        }
    })
    .await
}

pub(crate) async fn update_session<F>(session_id: &str, mut f: F) -> Result<(), String>
where
    F: FnMut(&mut AgentSession),
{
    super::session_store::validate_session_id(session_id)?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    f(&mut session);
    super::session_store::save(&session).await
}

pub(crate) fn find_run(session: &AgentSession, request_id: &str) -> Option<usize> {
    session
        .diagnostic_runs
        .iter()
        .position(|run| run.request_id == request_id)
}

pub(crate) fn trim<T>(items: &mut Vec<T>, max: usize) {
    while items.len() > max {
        items.remove(0);
    }
}

pub(crate) fn clip(value: &str) -> String {
    let clean = value.replace(['\n', '\r', '\t'], " ");
    let mut end = clean.len();
    for (count, (idx, _)) in clean.char_indices().enumerate() {
        if count == MAX_TEXT {
            end = idx;
            break;
        }
    }
    let mut out = clean[..end].to_string();
    if clean[end..].chars().next().is_some() {
        out.push_str("...");
    }
    out
}

fn safe_summary(run: &AgentDiagnosticRun, error_type: &str) -> String {
    if let Some(tool) = &run.last_tool {
        return clip(&format!(
            "Interruption pendant le tool {} ({error_type}).",
            tool.name
        ));
    }
    clip(&format!(
        "Interruption pendant {} ({error_type}).",
        run.phase
    ))
}

fn safe_code(message: &str) -> String {
    let code = classify_error(message, message == "ollama_connection_lost");
    if code == "unknown" {
        "stream_error".to_string()
    } else {
        code
    }
}

fn classify_error(message: &str, is_connection: bool) -> String {
    if is_connection || message == "ollama_connection_lost" {
        return "connection_lost".to_string();
    }
    if message.contains("Timeout") || message.contains("timeout") {
        return "timeout".to_string();
    }
    if message.contains("Limite de tours") {
        return "max_turns".to_string();
    }
    if message.contains("répété") || message.contains("circuit") {
        return "circuit_breaker".to_string();
    }
    if message.contains("HTTP") || message.contains("rate") || message.contains("auth") {
        return "provider_error".to_string();
    }
    "unknown".to_string()
}
