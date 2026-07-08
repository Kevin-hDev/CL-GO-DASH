use super::types_ollama::ChatMessage;
use super::types_session::AgentSession;

pub const SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX: &str = "Subagent orchestration context:";

pub fn replace_gate_context(
    messages: &mut Vec<ChatMessage>,
    active: &[AgentSession],
    reports_injected: bool,
) {
    remove_gate_context(messages);
    if active.is_empty() {
        return;
    }
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: build_gate_content(active, reports_injected),
        ..Default::default()
    });
}

pub fn remove_gate_context(messages: &mut Vec<ChatMessage>) {
    messages.retain(|message| {
        !message
            .content
            .trim_start()
            .starts_with(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX)
    });
}

pub fn build_gate_content(active: &[AgentSession], reports_injected: bool) -> String {
    let items = active
        .iter()
        .map(format_active_subagent)
        .collect::<Vec<_>>()
        .join("\n");
    let report_state = if reports_injected {
        "New subagent reports were injected before this model call."
    } else {
        "No new subagent report was injected before this model call."
    };
    format!(
        "{SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX}\n\
         <subagent_final_gate final_answer_allowed=\"false\">\n\
         <instruction>Final answer is locked because current-turn subagents are still running. \
         Do not present conclusions, completion wording, or a full answer to the user. \
         You may call tools, inspect/wait/cancel/message subagents, or write at most a short progress update. \
         Keep the stream active until every current-turn subagent is finished and every required report has been injected.</instruction>\n\
         <report_state>{}</report_state>\n\
         <active_subagents>\n{}\n</active_subagents>\n\
         </subagent_final_gate>",
        xml(report_state),
        items
    )
}

fn format_active_subagent(session: &AgentSession) -> String {
    let activity = session
        .subagent_last_activity
        .as_ref()
        .map(|activity| {
            format!(
                "<last_activity kind=\"{}\" label=\"{}\">{}</last_activity>",
                xml(&activity.kind),
                xml(&activity.label),
                xml(activity.detail.as_deref().unwrap_or(""))
            )
        })
        .unwrap_or_else(|| "<last_activity />".to_string());
    format!(
        "<subagent id=\"{}\" name=\"{}\" type=\"{}\" status=\"{}\"><description>{}</description>{}</subagent>",
        xml(&session.id),
        xml(&session.name),
        xml(session.subagent_type.as_deref().unwrap_or("explorer")),
        xml(session.subagent_status.as_deref().unwrap_or("running")),
        xml(session.subagent_description.as_deref().unwrap_or("")),
        activity
    )
}

fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_session::{AgentSession, SubagentLastActivity};
    use chrono::Utc;

    #[test]
    fn gate_blocks_final_answer_until_reports() {
        let mut session = empty_subagent("child");
        session.subagent_last_activity = Some(SubagentLastActivity {
            kind: "tool".into(),
            label: "bash démarré".into(),
            detail: Some("sleep 10".into()),
            updated_at: Utc::now(),
        });

        let content = build_gate_content(&[session], false);

        assert!(content.starts_with(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX));
        assert!(content.contains("final_answer_allowed=\"false\""));
        assert!(content.contains("Final answer is locked"));
        assert!(content.contains("bash démarré"));
    }

    #[test]
    fn gate_escapes_subagent_fields() {
        let mut session = empty_subagent("child<&");
        session.name = "Gemini\"tor".into();
        session.subagent_description = Some("<analyse>".into());

        let content = build_gate_content(&[session], true);

        assert!(content.contains("id=\"child&lt;&amp;\""));
        assert!(content.contains("name=\"Gemini&quot;tor\""));
        assert!(content.contains("&lt;analyse&gt;"));
        assert!(content.contains("New subagent reports were injected"));
    }

    #[test]
    fn replace_gate_removes_stale_gate() {
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

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "normal");
        assert!(messages[1].content.contains("id=\"child\""));
        assert!(!messages[1].content.contains("stale"));
    }

    #[test]
    fn replace_gate_removes_gate_when_no_active_subagent() {
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
}
