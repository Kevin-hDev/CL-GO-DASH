use serde_json::Value;

use super::tool_todo_parse::{optional_text, MAX_TODO_REASON_CHARS};
use super::types_todo::AgentTodoItem;
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_tools::ToolResult;

pub(crate) use super::tool_todo_parse::parse_todos;
pub(crate) use super::tool_todo_state::apply_todos_to_session;

pub async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let todos = match parse_todos(args) {
        Ok(items) => items,
        Err(err) => return ToolResult::err(err),
    };

    match save_with(session_id, |session| {
        apply_todos_to_session(session, todos.clone());
        session.todos.clone()
    })
    .await
    {
        Ok(active) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list mise à jour.")
        }
        Err(_) => ToolResult::err("Mise à jour de la todo impossible."),
    }
}

pub async fn execute_history(_args: &Value, session_id: &str) -> ToolResult {
    match load_session(session_id).await {
        Ok(session) => ToolResult::ok(super::tool_todo_summary::history_summary(&session)),
        Err(_) => ToolResult::err("Historique todo indisponible."),
    }
}

pub async fn execute_pause(args: &Value, session_id: &str) -> ToolResult {
    let reason = match optional_text(args, "reason", MAX_TODO_REASON_CHARS) {
        Ok(value) => value,
        Err(err) => return ToolResult::err(err),
    };
    match save_with(session_id, |session| {
        super::tool_todo_state::pause_active(session, reason.clone());
        session.todos.clone()
    })
    .await
    {
        Ok(active) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list mise de côté.")
        }
        Err(_) => ToolResult::err("Mise en pause de la todo impossible."),
    }
}

pub async fn execute_resume(args: &Value, session_id: &str) -> ToolResult {
    let Some(run_id) = args.get("id").and_then(Value::as_str) else {
        return ToolResult::err("paramètre 'id' requis");
    };
    match save_with(session_id, |session| {
        super::tool_todo_state::resume_run(session, run_id)
    })
    .await
    {
        Ok(Ok(active)) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list reprise.")
        }
        Ok(Err(err)) => ToolResult::err(err),
        Err(_) => ToolResult::err("Reprise de la todo impossible."),
    }
}

pub async fn execute_delete(args: &Value, session_id: &str) -> ToolResult {
    let Some(run_id) = args.get("id").and_then(Value::as_str) else {
        return ToolResult::err("paramètre 'id' requis");
    };
    match save_with(session_id, |session| {
        super::tool_todo_state::delete_run(session, run_id)
    })
    .await
    {
        Ok(Ok(active)) => {
            emit_update(session_id, active);
            ToolResult::ok("Todo list supprimée.")
        }
        Ok(Err(err)) => ToolResult::err(err),
        Err(_) => ToolResult::err("Suppression de la todo impossible."),
    }
}

pub async fn append_session_reminder(
    messages: &mut [super::types_ollama::ChatMessage],
    session_id: &str,
) {
    let Ok(session) = load_session(session_id).await else {
        return;
    };
    let Some(reminder) = super::tool_todo_summary::reminder(&session) else {
        return;
    };
    if let Some(system) = messages
        .first_mut()
        .filter(|message| message.role == "system")
    {
        system.content.push_str(&reminder);
    }
}

pub(crate) fn emit_update(session_id: &str, todos: Vec<AgentTodoItem>) {
    let Some(app) = super::app_handle_global::get() else {
        return;
    };
    let emitter = super::stream_events::AgentEventEmitter::new(app.clone(), session_id.to_string());
    let _ = emitter.send(StreamEvent::TodoUpdated { todos });
}

async fn save_with<T>(
    session_id: &str,
    edit: impl FnOnce(&mut super::types_session::AgentSession) -> T,
) -> Result<T, String> {
    super::session_store::validate_session_id(session_id)?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    let result = edit(&mut session);
    super::session_store::save(&session).await?;
    Ok(result)
}

async fn load_session(session_id: &str) -> Result<super::types_session::AgentSession, String> {
    super::session_store::validate_session_id(session_id)?;
    super::session_store::get(session_id).await
}

#[cfg(test)]
#[path = "tool_todo_tests.rs"]
mod tests;
