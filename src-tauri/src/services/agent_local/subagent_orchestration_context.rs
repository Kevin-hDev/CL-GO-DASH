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
    let context = ChatMessage {
        role: "system".to_string(),
        content: build_gate_content(active, reports_injected),
        ..Default::default()
    };
    super::subagent_report_context::insert_leading_system_message(messages, context);
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
        "Terminal reports are available in this request."
    } else {
        "No terminal report is available in this request."
    };
    format!(
        "{SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX}\n\
         <subagent_runtime_context>\n\
         <guidance>Some subagents are still working. Continue useful independent work without \
         duplicating delegated work. Terminal reports arrive automatically. If a terminal report \
         is available while other subagents are still working, use it for useful independent work \
         and defer the overall summary until every active subagent has reported. When no useful \
         independent work remains, give at most one short progress update and finish this turn \
         without a tool call.</guidance>\n\
         <report_state>{}</report_state>\n\
         <active_subagents>\n{}\n</active_subagents>\n\
         </subagent_runtime_context>",
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
#[path = "subagent_orchestration_context_tests.rs"]
mod tests;
