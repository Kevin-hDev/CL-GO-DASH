use super::*;
use chrono::Utc;

fn message(id: &str, role: &str, content: &str) -> AgentMessage {
    AgentMessage {
        id: id.into(),
        role: role.into(),
        content: content.into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    }
}

fn session() -> AgentSession {
    AgentSession {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        name: "Main".into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: vec![
            message("m1", "user", "start"),
            message("m2", "assistant", "answer"),
            message("m3", "user", "future"),
        ],
        todos: vec![],
        todo_neglect_count: 0,
        todo_runs: vec![],
        active_todo_run_id: None,
        stream_failures: vec![],
        diagnostic_runs: vec![],
        plan_mode_enabled: false,
        plan_runs: vec![],
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        plan_approval_decision: None,
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
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
    }
}

#[test]
fn build_clone_cuts_at_selected_message() {
    let source = session();
    let clone = build_clone(&source, "m2", CloneMode::Cut, 1);

    assert_eq!(clone.messages.len(), 2);
    assert_eq!(clone.messages[1].id, "m2");
    assert_eq!(clone.clone_parent_session_id, Some(source.id));
    assert_eq!(clone.clone_parent_message_id, Some("m2".into()));
    assert_eq!(clone.clone_mode, Some(CloneMode::Cut));
    assert!(clone.clone_summary.is_none());
    assert!(clone.stream_failures.is_empty());
    assert!(clone.diagnostic_runs.is_empty());
}

#[test]
fn hidden_context_message_uses_clone_prefix() {
    let hidden = hidden_context_message("Useful summary");

    assert_eq!(hidden.role, "user");
    assert!(hidden.content.starts_with(clone_summary::CLONE_SUMMARY_PREFIX));
}
