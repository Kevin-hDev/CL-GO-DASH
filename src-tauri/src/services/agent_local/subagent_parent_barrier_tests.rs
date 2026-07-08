use super::*;
use crate::services::agent_local::types_session::{AgentSession, SubagentLastActivity};
use chrono::Utc;

#[test]
fn reminder_is_due_immediately_then_after_interval() {
    let now = Instant::now();
    assert!(should_emit_reminder(false, None, now));
    assert!(!should_emit_reminder(true, Some(now), now));
    assert!(should_emit_reminder(
        true,
        Some(now - REMINDER_INTERVAL),
        now
    ));
}

#[test]
fn reminder_blocks_final_answer_until_reports() {
    let mut session = empty_subagent("child");
    session.subagent_last_activity = Some(SubagentLastActivity {
        kind: "tool".into(),
        label: "bash démarré".into(),
        detail: Some("sleep 10".into()),
        updated_at: Utc::now(),
    });

    let content = build_reminder_content(&[session]);

    assert!(content.starts_with(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX));
    assert!(content.contains("Do not write a final answer yet"));
    assert!(content.contains("bash démarré"));
}

fn empty_subagent(id: &str) -> AgentSession {
    AgentSession {
        id: id.into(),
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
        subagent_status: Some(super::super::subagent_status::RUNNING.into()),
        subagent_run_id: None,
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
