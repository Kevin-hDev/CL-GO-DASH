use super::types_session::{AgentSession, AgentSessionMeta};

pub(super) fn format_meta(meta: AgentSessionMeta) -> String {
    let activity = meta
        .subagent_last_activity
        .as_ref()
        .map(|activity| text_field(&activity.label))
        .unwrap_or_default();
    format!(
        "- id={} name=\"{}\" type={} status={} run_id=\"{}\" description=\"{}\" last_activity=\"{}\"",
        meta.id,
        text_field(&meta.name),
        meta.subagent_type.unwrap_or_else(|| "explorer".to_string()),
        meta.subagent_status
            .unwrap_or_else(|| "completed".to_string()),
        text_field(&meta.subagent_run_id.unwrap_or_default()),
        text_field(&meta.subagent_description.unwrap_or_default()),
        activity
    )
}

pub(super) fn format_children(children: &[AgentSession]) -> String {
    children
        .iter()
        .map(format_child)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn format_child(child: &AgentSession) -> String {
    let activity = child
        .subagent_last_activity
        .as_ref()
        .map(|activity| {
            format!(
                "<last_activity kind=\"{}\" label=\"{}\">{}</last_activity>",
                xml_attr(&activity.kind),
                xml_attr(&activity.label),
                xml_text(activity.detail.as_deref().unwrap_or(""))
            )
        })
        .unwrap_or_else(|| "<last_activity />".to_string());
    format!(
        "<subagent id=\"{}\" name=\"{}\" type=\"{}\" status=\"{}\" run_id=\"{}\" queued_prompts=\"{}\">\n<description>{}</description>\n{}\n<summary>{}</summary>\n</subagent>",
        xml_attr(&child.id),
        xml_attr(&child.name),
        xml_attr(child.subagent_type.as_deref().unwrap_or("explorer")),
        xml_attr(child.subagent_status.as_deref().unwrap_or("completed")),
        xml_attr(child.subagent_run_id.as_deref().unwrap_or("")),
        child.subagent_queued_prompts.len(),
        xml_text(child.subagent_description.as_deref().unwrap_or("")),
        activity,
        xml_text(child.subagent_summary.as_deref().unwrap_or(""))
    )
}

fn text_field(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn xml_attr(value: &str) -> String {
    text_field(value)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
