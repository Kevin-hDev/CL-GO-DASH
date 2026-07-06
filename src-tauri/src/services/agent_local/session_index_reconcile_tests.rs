use super::*;
use chrono::Utc;
use tempfile::TempDir;

fn session(id: &str) -> AgentSession {
    AgentSession {
        id: id.into(),
        name: "Clone - CLONE".into(),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        archived_at: Some(Utc::now()),
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: vec![],
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
        project_id: Some("project".into()),
        working_dir: "/tmp/project".into(),
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        clone_parent_session_id: Some("parent".into()),
        clone_parent_message_id: Some("message".into()),
        clone_mode: Some(crate::services::agent_local::types_session::CloneMode::Summary),
        clone_summary: Some("summary".into()),
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: None,
        git_branch: None,
    }
}

#[tokio::test]
async fn reconcile_rebuilds_stale_active_clone_index() {
    let tmp = TempDir::new().unwrap();
    let session = session("clone-1");
    let stale = AgentSessionMeta {
        archived_at: None,
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        ..meta_from_session(&session)
    };
    tokio::fs::write(
        tmp.path().join("clone-1.json"),
        serde_json::to_string_pretty(&session).unwrap(),
    )
    .await
    .unwrap();

    let entries = reconcile_index(&tmp.path().join("index.json"), vec![stale])
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].archived_at, session.archived_at);
    assert_eq!(
        entries[0].clone_parent_session_id,
        session.clone_parent_session_id
    );
    assert_eq!(
        entries[0].clone_parent_message_id,
        session.clone_parent_message_id
    );
}
