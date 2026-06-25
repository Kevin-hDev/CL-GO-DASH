use chrono::Utc;

use super::types_diagnostics::AgentStreamFailure;
use super::types_session::AgentSession;

const MAX_STREAM_FAILURES: usize = 20;

pub async fn record_failure(session_id: &str, message: &str, is_connection: bool) {
    if super::session_store::validate_session_id(session_id).is_err() {
        return;
    }
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let Ok(mut session) = super::session_store::get(session_id).await else {
        return;
    };
    push_failure(&mut session, message, is_connection);
    let _ = super::session_store::save(&session).await;
}

pub async fn diagnostics_text(session_id: &str) -> Result<String, String> {
    super::session_store::validate_session_id(session_id)?;
    let session = super::session_store::get(session_id).await?;
    Ok(format_diagnostics(&session))
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
    while session.stream_failures.len() > MAX_STREAM_FAILURES {
        session.stream_failures.remove(0);
    }
}

fn format_diagnostics(session: &AgentSession) -> String {
    if session.stream_failures.is_empty() {
        return "Aucun diagnostic de stream enregistré.".to_string();
    }
    session
        .stream_failures
        .iter()
        .rev()
        .map(|failure| {
            let todo = failure
                .active_todo_title
                .as_ref()
                .map(|title| format!(" todo=\"{title}\""))
                .unwrap_or_default();
            format!(
                "- at={} code={} connection={}{}",
                failure.occurred_at.to_rfc3339(),
                failure.code,
                failure.is_connection,
                todo
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn safe_code(message: &str) -> String {
    if message == "ollama_connection_lost" {
        return "ollama_connection_lost".to_string();
    }
    if message.contains("Timeout") {
        return "stream_timeout".to_string();
    }
    if message.contains("Limite de tours") {
        return "max_turns_reached".to_string();
    }
    if message.contains("Ollama HTTP") || message.contains("Erreur serveur Ollama") {
        return "ollama_server_error".to_string();
    }
    if message == "model_not_found" || message == "rate_limit" || message == "auth_failed" {
        return message.to_string();
    }
    "stream_error".to_string()
}
