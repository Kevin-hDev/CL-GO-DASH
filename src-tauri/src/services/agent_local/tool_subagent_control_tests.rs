use super::*;
use crate::services::agent_local::types_session::AgentSession;
use chrono::Utc;
use serde_json::json;

#[test]
fn running_child_has_pending_work() {
    let mut child = child("running");

    assert!(child_has_pending_work(&child));

    child.subagent_status = Some("completed".into());
    assert!(!child_has_pending_work(&child));
}

#[test]
fn queued_prompt_keeps_completed_child_active_for_wait() {
    let mut child = child("completed");
    child.subagent_queued_prompts.push("suite".into());

    assert!(child_has_pending_work(&child));
}

#[test]
fn wait_subagent_rejects_too_many_ids() {
    let ids = (0..=MAX_WAIT_SUBAGENT_IDS)
        .map(|idx| json!(format!("child-{idx}")))
        .collect::<Vec<_>>();

    let result = subagent_ids(&json!({ "subagent_ids": ids }));

    assert!(result.is_err());
}

fn child(status: &str) -> AgentSession {
    AgentSession {
        id: "child".into(),
        name: "Geminitor".into(),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: Vec::new(),
        todos: Vec::new(),
        todo_neglect_count: 0,
        todo_runs: Vec::new(),
        active_todo_run_id: None,
        stream_failures: Vec::new(),
        diagnostic_runs: Vec::new(),
        plan_mode_enabled: false,
        plan_runs: Vec::new(),
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        plan_approval_decision: None,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        parent_session_id: Some("parent".into()),
        subagent_type: Some("explorer".into()),
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: Some(status.into()),
        subagent_run_id: Some("run-1".into()),
        subagent_description: Some("Analyse".into()),
        subagent_color_key: Some("geminitor".into()),
        subagent_summary: None,
        subagent_last_activity: None,
        subagent_queued_prompts: Vec::new(),
        subagent_hidden_reports: Vec::new(),
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        clone_root_session_id: None,
        git_branch: None,
    }
}
