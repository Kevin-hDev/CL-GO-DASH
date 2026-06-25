use super::stream_diagnostics_support::{apply_failure, event, push_event};
use super::types_diagnostics::AgentDiagnosticRun;
use super::types_session::AgentSession;
use chrono::Utc;

#[test]
fn diagnostic_events_are_bounded() {
    let mut run = test_run();
    for i in 0..20 {
        push_event(&mut run, "tool_execution", &format!("event {i}"), None, None);
    }
    assert_eq!(run.events.len(), 12);
    assert_eq!(run.events[0].message, "event 8");
}

#[test]
fn failure_summary_uses_last_tool_without_raw_error() {
    let mut session = test_session();
    session.diagnostic_runs.push(test_run());
    session.diagnostic_runs[0].last_tool = Some(super::types_diagnostics::AgentDiagnosticTool {
        name: "write_file".to_string(),
        status: "started".to_string(),
        args: None,
        is_error: false,
    });
    apply_failure(&mut session, 0, "/Users/kevinh/secret stack trace", false);
    let run = &session.diagnostic_runs[0];
    assert_eq!(run.status, "failed");
    assert_eq!(run.error_type.as_deref(), Some("unknown"));
    assert_eq!(
        run.safe_summary.as_deref(),
        Some("Interruption pendant le tool write_file (unknown).")
    );
}

fn test_run() -> AgentDiagnosticRun {
    let now = Utc::now();
    AgentDiagnosticRun {
        request_id: "req-1".to_string(),
        generation: 1,
        status: "running".to_string(),
        severity: "info".to_string(),
        started_at: now,
        updated_at: now,
        ended_at: None,
        phase: "request_start".to_string(),
        error_type: None,
        last_tool: None,
        active_todo: None,
        safe_summary: None,
        events: vec![event("request_start", "start", None, None)],
    }
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
        diagnostic_runs: vec![],
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
