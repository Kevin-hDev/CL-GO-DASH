use super::*;
use crate::services::agent_local::types_session::{AgentSession, SubagentLastActivity};
use chrono::Utc;

#[test]
fn context_lists_active_subagent_activity() {
    let mut session = empty_subagent("child");
    session.subagent_last_activity = Some(SubagentLastActivity {
        kind: "tool".into(),
        label: "bash démarré".into(),
        detail: Some("sleep 10".into()),
        updated_at: Utc::now(),
    });

    let content = build_gate_content(&[session], false);

    assert!(content.starts_with(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX));
    assert!(content.contains("<subagent_runtime_context>"));
    assert!(content.contains("bash démarré"));
}

#[test]
fn context_escapes_subagent_fields() {
    let mut session = empty_subagent("child<&");
    session.name = "Gemini\"tor".into();
    session.subagent_description = Some("<analyse>".into());

    let content = build_gate_content(&[session], true);

    assert!(content.contains("id=\"child&lt;&amp;\""));
    assert!(content.contains("name=\"Gemini&quot;tor\""));
    assert!(content.contains("&lt;analyse&gt;"));
    assert!(content.contains("Terminal reports are available"));
}

#[test]
fn replace_context_is_unique_and_stays_in_the_leading_system_block() {
    let mut messages = vec![
        ChatMessage {
            role: "user".into(),
            content: "normal".into(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".into(),
            content: format!("{SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX}\nstale"),
            ..Default::default()
        },
    ];

    replace_gate_context(&mut messages, &[empty_subagent("child")], false);
    replace_gate_context(&mut messages, &[empty_subagent("child")], false);

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "system");
    assert!(messages[0].content.contains("id=\"child\""));
    assert!(!messages[0].content.contains("stale"));
    assert_eq!(messages[1].content, "normal");
    assert_eq!(
        messages
            .iter()
            .filter(|message| message.content.starts_with(
                SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX
            ))
            .count(),
        1
    );
}

#[test]
fn replace_context_removes_context_when_no_subagent_is_active() {
    let mut messages = vec![
        ChatMessage {
            role: "user".into(),
            content: "normal".into(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".into(),
            content: format!("{SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX}\nstale"),
            ..Default::default()
        },
    ];

    replace_gate_context(&mut messages, &[], false);

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "normal");
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
