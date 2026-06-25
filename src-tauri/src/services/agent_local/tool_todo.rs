use serde_json::Value;

use super::types_todo::{AgentTodoItem, AgentTodoStatus};
use crate::services::agent_local::types_ollama::StreamEvent;
use crate::services::agent_local::types_tools::ToolResult;

const MAX_TODOS: usize = 50;
const MAX_TODO_TEXT_CHARS: usize = 180;

pub async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let todos = match parse_todos(args) {
        Ok(items) => items,
        Err(err) => return ToolResult::err(err),
    };

    match save_todos(session_id, todos.clone()).await {
        Ok(()) => {
            emit_update(session_id, todos);
            ToolResult::ok("Todo list mise à jour.")
        }
        Err(_) => ToolResult::err("Mise à jour de la todo impossible."),
    }
}

pub(crate) fn parse_todos(args: &Value) -> Result<Vec<AgentTodoItem>, String> {
    let raw = args
        .get("todos")
        .and_then(Value::as_array)
        .ok_or_else(|| "paramètre 'todos' requis".to_string())?;
    if raw.len() > MAX_TODOS {
        return Err(format!("maximum {MAX_TODOS} tâches"));
    }

    let mut in_progress = 0usize;
    let mut todos = Vec::with_capacity(raw.len());
    for item in raw {
        let obj = item
            .as_object()
            .ok_or_else(|| "chaque tâche doit être un objet JSON".to_string())?;
        let content = clean_text(obj.get("content"), "content")?;
        let active_form = match obj.get("active_form").or_else(|| obj.get("activeForm")) {
            Some(Value::Null) | None => None,
            value => Some(clean_text(value, "active_form")?),
        };
        let status = parse_status(obj.get("status"))?;
        if status == AgentTodoStatus::InProgress {
            in_progress += 1;
        }
        todos.push(AgentTodoItem {
            content,
            active_form,
            status,
        });
    }

    if in_progress > 1 {
        return Err("une seule tâche peut être en cours".to_string());
    }
    Ok(todos)
}

pub(crate) fn apply_todos_to_session(
    session: &mut super::types_session::AgentSession,
    todos: Vec<AgentTodoItem>,
) {
    session.todos = todos;
}

async fn save_todos(session_id: &str, todos: Vec<AgentTodoItem>) -> Result<(), String> {
    super::session_store::validate_session_id(session_id)?;
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    apply_todos_to_session(&mut session, todos);
    super::session_store::save(&session).await
}

pub(crate) fn emit_update(session_id: &str, todos: Vec<AgentTodoItem>) {
    let Some(app) = super::app_handle_global::get() else {
        return;
    };
    let emitter = super::stream_events::AgentEventEmitter::new(app.clone(), session_id.to_string());
    let _ = emitter.send(StreamEvent::TodoUpdated { todos });
}

fn clean_text(value: Option<&Value>, name: &str) -> Result<String, String> {
    let text = value
        .and_then(Value::as_str)
        .ok_or_else(|| format!("'{name}' doit être une chaîne"))?
        .trim();
    if text.is_empty() {
        return Err(format!("'{name}' ne peut pas être vide"));
    }
    if text.chars().count() > MAX_TODO_TEXT_CHARS {
        return Err(format!("'{name}' est trop long"));
    }
    if text.chars().any(|ch| ch.is_control()) {
        return Err(format!("'{name}' contient des caractères invalides"));
    }
    Ok(text.to_string())
}

fn parse_status(value: Option<&Value>) -> Result<AgentTodoStatus, String> {
    match value.and_then(Value::as_str) {
        Some("pending") => Ok(AgentTodoStatus::Pending),
        Some("in_progress") => Ok(AgentTodoStatus::InProgress),
        Some("completed") => Ok(AgentTodoStatus::Completed),
        _ => Err("'status' doit valoir pending, in_progress ou completed".to_string()),
    }
}

#[cfg(test)]
#[path = "tool_todo_tests.rs"]
mod tests;
