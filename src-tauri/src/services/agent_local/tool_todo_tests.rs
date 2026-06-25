use super::*;
use crate::services::agent_local::types_session::AgentSession;
use chrono::Utc;
use serde_json::json;

#[test]
fn parse_accepts_valid_todos() {
    let parsed = parse_todos(&json!({
        "todos": [
            {"content": "Lire le code", "status": "completed"},
            {"content": "Implémenter", "activeForm": "Implémente", "status": "in_progress"},
            {"content": "Tester", "status": "pending"}
        ]
    }))
    .unwrap();

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[1].active_form.as_deref(), Some("Implémente"));
    assert_eq!(parsed[1].status, AgentTodoStatus::InProgress);
}

#[test]
fn parse_rejects_invalid_status() {
    let err = parse_todos(&json!({
        "todos": [{"content": "Lire", "status": "started"}]
    }))
    .unwrap_err();

    assert!(err.contains("status"));
}

#[test]
fn parse_rejects_more_than_fifty_todos() {
    let todos: Vec<_> = (0..51)
        .map(|i| json!({"content": format!("Tâche {i}"), "status": "pending"}))
        .collect();
    let err = parse_todos(&json!({ "todos": todos })).unwrap_err();

    assert!(err.contains("maximum 50"));
}

#[test]
fn parse_rejects_multiple_in_progress() {
    let err = parse_todos(&json!({
        "todos": [
            {"content": "A", "status": "in_progress"},
            {"content": "B", "status": "in_progress"}
        ]
    }))
    .unwrap_err();

    assert!(err.contains("une seule"));
}

#[test]
fn apply_todos_updates_session() {
    let mut session = test_session();
    let todos = parse_todos(&json!({
        "todos": [{"content": "Valider", "status": "completed"}]
    }))
    .unwrap();

    apply_todos_to_session(&mut session, todos);

    assert_eq!(session.todos.len(), 1);
    assert_eq!(session.todos[0].content, "Valider");
}

fn test_session() -> AgentSession {
    AgentSession {
        id: "abc-123".into(),
        name: "Test".into(),
        created_at: Utc::now(),
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: vec![],
        todos: vec![],
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
    }
}
