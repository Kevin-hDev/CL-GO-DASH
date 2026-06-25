use super::types_diagnostics::AgentDiagnosticTool;
use super::types_session::AgentSession;

pub(crate) const MAX_DIAGNOSTIC_TOOLS: usize = 20;
pub(crate) const DEFAULT_TOOL_LIMIT: usize = 1;

pub(crate) fn bounded_tool_limit(limit: usize) -> usize {
    limit.clamp(1, MAX_DIAGNOSTIC_TOOLS)
}

pub(crate) fn recent_relevant_tools(
    session: &AgentSession,
    limit: usize,
) -> Vec<AgentDiagnosticTool> {
    let limit = bounded_tool_limit(limit);
    session
        .diagnostic_runs
        .iter()
        .rev()
        .flat_map(|run| run.recent_tools.iter().rev())
        .filter(|tool| !is_internal_diagnostic_tool(&tool.name))
        .take(limit)
        .cloned()
        .collect()
}

fn is_internal_diagnostic_tool(name: &str) -> bool {
    matches!(name, "agent_diagnostics")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::stream_diagnostics_support::event;
    use crate::services::agent_local::types_diagnostics::AgentDiagnosticRun;
    use chrono::Utc;

    #[test]
    fn recent_tools_skip_agent_diagnostics_and_respect_limit() {
        let mut session = test_session();
        session.diagnostic_runs.push(test_run(vec![
            tool("write_file", "completed"),
            tool("agent_diagnostics", "started"),
            tool("grep", "completed"),
        ]));

        let tools = recent_relevant_tools(&session, 2);

        assert_eq!(
            tools
                .iter()
                .map(|tool| tool.name.as_str())
                .collect::<Vec<_>>(),
            vec!["grep", "write_file"]
        );
    }

    #[test]
    fn tool_limit_is_bounded() {
        assert_eq!(bounded_tool_limit(0), 1);
        assert_eq!(bounded_tool_limit(100), 20);
    }

    fn test_run(recent_tools: Vec<AgentDiagnosticTool>) -> AgentDiagnosticRun {
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
            last_tool: recent_tools.last().cloned(),
            recent_tools,
            active_todo: None,
            safe_summary: None,
            events: vec![event("request_start", "start", None, None)],
        }
    }

    fn tool(name: &str, status: &str) -> AgentDiagnosticTool {
        AgentDiagnosticTool {
            name: name.to_string(),
            status: status.to_string(),
            args: None,
            is_error: false,
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
}
