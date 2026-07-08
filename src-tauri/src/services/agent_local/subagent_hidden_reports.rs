use super::types_ollama::ChatMessage;
use super::types_session::SubagentHiddenReport;
use chrono::Utc;
use std::collections::BTreeSet;
use uuid::Uuid;

const MAX_PENDING_REPORTS: usize = 16;
const MAX_REPORT_SUMMARY_CHARS: usize = 12_000;
pub const SUBAGENT_REPORT_CONTEXT_PREFIX: &str = "Subagent report context:";

pub async fn append(parent_id: &str, report: SubagentHiddenReport) -> Result<(), String> {
    let mut session = super::session_store::get(parent_id).await?;
    if session
        .subagent_hidden_reports
        .iter()
        .any(|seen| is_same_report(seen, &report))
    {
        return Ok(());
    }
    session.subagent_hidden_reports.push(report);
    while session.subagent_hidden_reports.len() > MAX_PENDING_REPORTS {
        session.subagent_hidden_reports.remove(0);
    }
    super::session_store::save(&session).await
}

pub async fn take_for_context(session_id: &str) -> Vec<ChatMessage> {
    let Ok(mut session) = super::session_store::get(session_id).await else {
        return Vec::new();
    };
    if session.subagent_hidden_reports.is_empty() {
        return Vec::new();
    }
    let reports = std::mem::take(&mut session.subagent_hidden_reports);
    if super::session_store::save(&session).await.is_err() {
        return Vec::new();
    }
    reports.into_iter().map(report_to_message).collect()
}

pub async fn has_pending_except(session_id: &str, ignored_child_ids: &BTreeSet<String>) -> bool {
    super::session_store::get(session_id)
        .await
        .map(|session| {
            session
                .subagent_hidden_reports
                .iter()
                .any(|report| !ignored_child_ids.contains(&report.child_session_id))
        })
        .unwrap_or(false)
}

pub fn build_report(
    child_session_id: String,
    name: String,
    subagent_type: String,
    status: String,
    summary: String,
) -> SubagentHiddenReport {
    SubagentHiddenReport {
        id: Uuid::new_v4().to_string(),
        child_session_id,
        name,
        subagent_type,
        status,
        summary: truncate_chars(&summary, MAX_REPORT_SUMMARY_CHARS),
        created_at: Utc::now(),
    }
}

fn report_to_message(report: SubagentHiddenReport) -> ChatMessage {
    let summary = escape_xml(report.summary.trim());
    ChatMessage {
        role: "user".to_string(),
        content: format!(
            "{SUBAGENT_REPORT_CONTEXT_PREFIX}\n\
             <subagent id=\"{}\" name=\"{}\" type=\"{}\" status=\"{}\">\n\
             <summary>\n{}\n</summary>\n\
             </subagent>",
            escape_xml(&report.child_session_id),
            escape_xml(&report.name),
            escape_xml(&report.subagent_type),
            escape_xml(&report.status),
            summary
        ),
        ..Default::default()
    }
}

fn is_same_report(seen: &SubagentHiddenReport, report: &SubagentHiddenReport) -> bool {
    seen.id == report.id
        || (seen.child_session_id == report.child_session_id
            && seen.status == report.status
            && seen.summary == report.summary)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut output = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        output.push_str("\n[rapport tronqué]");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_context_contains_subagent_id() {
        let report = build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "Résumé".into(),
        );
        let message = report_to_message(report);
        assert!(message.content.starts_with(SUBAGENT_REPORT_CONTEXT_PREFIX));
        assert!(message.content.contains("id=\"child\""));
        assert!(message.content.contains("Résumé"));
    }

    #[test]
    fn same_child_status_and_summary_counts_as_duplicate() {
        let first = build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "Résumé".into(),
        );
        let mut second = first.clone();
        second.id = "another-id".into();
        assert!(is_same_report(&first, &second));
    }

    #[test]
    fn build_report_truncates_large_summary() {
        let report = build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "x".repeat(MAX_REPORT_SUMMARY_CHARS + 10),
        );
        assert_eq!(
            report.summary.chars().count(),
            MAX_REPORT_SUMMARY_CHARS + "\n[rapport tronqué]".chars().count()
        );
        assert!(report.summary.contains("[rapport tronqué]"));
    }
}
