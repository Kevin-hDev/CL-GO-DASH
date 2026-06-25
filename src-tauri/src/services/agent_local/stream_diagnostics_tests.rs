use super::stream_diagnostics::push_failure;
use super::types_session::AgentSession;
use chrono::Utc;

#[test]
fn stream_failures_are_bounded_and_sanitized() {
    let mut session = test_session();
    for i in 0..25 {
        push_failure(
            &mut session,
            &format!("secret /Users/kevinh/project token-{i}"),
            false,
        );
    }

    assert_eq!(session.stream_failures.len(), 20);
    assert!(session
        .stream_failures
        .iter()
        .all(|failure| failure.code == "stream_error"));
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
        todo_runs: vec![],
        active_todo_run_id: None,
        stream_failures: vec![],
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
