use super::tool_todo_state::completed_count;
use super::types_session::AgentSession;
use super::types_todo::{AgentTodoRun, AgentTodoRunStatus};

pub fn history_summary(session: &AgentSession) -> String {
    if session.todo_runs.is_empty() {
        return "Aucune todo list sauvegardée.".to_string();
    }
    session
        .todo_runs
        .iter()
        .rev()
        .map(format_run_summary)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn reminder(session: &AgentSession) -> Option<String> {
    let paused = session
        .todo_runs
        .iter()
        .rev()
        .find(|run| run.status == AgentTodoRunStatus::Paused);
    let failure = session.stream_failures.last();
    if paused.is_none() && failure.is_none() {
        return None;
    }

    let mut lines = vec!["\n\n## Todo memory".to_string()];
    if let Some(run) = paused {
        lines.push(format!(
            "A paused todo exists: {} ({}/{} done). Use todo_history and todo_resume when relevant.",
            run.title,
            completed_count(&run.todos),
            run.todos.len()
        ));
    }
    if let Some(err) = failure {
        lines.push(format!(
            "Last stream diagnostic: {}. Use agent_diagnostics if you need details.",
            err.code
        ));
    }
    Some(lines.join("\n"))
}

fn format_run_summary(run: &AgentTodoRun) -> String {
    let reason = run
        .paused_reason
        .as_ref()
        .map(|value| format!(" reason=\"{value}\""))
        .unwrap_or_default();
    format!(
        "- id={} status={:?} progress={}/{} title=\"{}\"{}",
        run.id,
        run.status,
        completed_count(&run.todos),
        run.todos.len(),
        run.title,
        reason
    )
}
