use super::stream_events::AgentEventEmitter;
use super::types_ollama::ChatMessage;
use super::types_session::AgentSession;
use super::types_stream::{StreamEvent, StreamResult};
use serde_json::json;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub const REMINDER_INTERVAL: Duration = Duration::from_secs(10 * 60);
const POLL_INTERVAL: Duration = Duration::from_secs(1);
pub const SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX: &str = "Subagent orchestration context:";

pub struct ParentSubagentOrchestrator {
    parent_session_id: String,
    initial_active: BTreeSet<String>,
    last_reminder_at: Option<Instant>,
    reminder_sent: bool,
}

impl ParentSubagentOrchestrator {
    pub async fn new(parent_session_id: &str) -> Self {
        let initial_active = active_set(parent_session_id).await;
        super::subagent_flow_log::record(
            "parent_orchestrator_created",
            Some(parent_session_id),
            None,
            None,
            json!({"initial_active": initial_active.len()}),
        );
        Self {
            parent_session_id: parent_session_id.to_string(),
            initial_active,
            last_reminder_at: None,
            reminder_sent: false,
        }
    }

    pub async fn inject_pending_reports(&mut self, messages: &mut Vec<ChatMessage>) -> bool {
        let reports =
            super::subagent_hidden_reports::take_for_context(&self.parent_session_id).await;
        if reports.is_empty() {
            return false;
        }
        let count = reports.len();
        messages.extend(reports);
        super::subagent_flow_log::record(
            "parent_orchestrator_reports_injected",
            Some(&self.parent_session_id),
            None,
            None,
            json!({"count": count}),
        );
        true
    }

    pub async fn finalize_content_phase(
        &self,
        on_event: &AgentEventEmitter,
        result: &StreamResult,
        plan_active: bool,
    ) {
        let awaiting_report = super::subagent_hidden_reports::has_pending_except(
            &self.parent_session_id,
            &self.initial_active,
        )
        .await;
        super::stream_buffer::finalize_content_phase(
            on_event,
            result,
            plan_active,
            awaiting_report || !self.current_turn_active().await.is_empty(),
        );
    }
    pub async fn continue_after_no_tool_turn(
        &mut self,
        on_event: &AgentEventEmitter,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
    ) -> Result<bool, String> {
        let should_continue = self.after_no_tool_turn(messages, cancel).await?;
        if should_continue {
            let _ = on_event.send(StreamEvent::TurnEnd {});
        }
        Ok(should_continue)
    }

    pub async fn after_no_tool_turn(
        &mut self,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
    ) -> Result<bool, String> {
        loop {
            if cancel.is_cancelled() {
                super::subagent_flow_log::record(
                    "parent_orchestrator_cancelled",
                    Some(&self.parent_session_id),
                    None,
                    None,
                    json!({}),
                );
                return Err("Annulé".to_string());
            }
            if self.inject_pending_reports(messages).await {
                return Ok(true);
            }
            let active = self.current_turn_active().await;
            if active.is_empty() {
                super::subagent_flow_log::record(
                    "parent_orchestrator_finish_allowed",
                    Some(&self.parent_session_id),
                    None,
                    None,
                    json!({}),
                );
                return Ok(false);
            }
            if should_emit_reminder(self.reminder_sent, self.last_reminder_at, Instant::now()) {
                messages.push(reminder_message(&active));
                self.reminder_sent = true;
                self.last_reminder_at = Some(Instant::now());
                super::subagent_flow_log::record(
                    "parent_orchestrator_reminder_injected",
                    Some(&self.parent_session_id),
                    None,
                    None,
                    json!({"active": active.len()}),
                );
                return Ok(true);
            }
            tokio::select! {
                _ = cancel.cancelled() => return Err("Annulé".to_string()),
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
            }
        }
    }

    async fn current_turn_active(&self) -> Vec<AgentSession> {
        current_turn_active(&self.parent_session_id, &self.initial_active).await
    }
}

async fn current_turn_active(
    parent_session_id: &str,
    initial_active: &BTreeSet<String>,
) -> Vec<AgentSession> {
    let ids = super::subagent_registry::active_children_for_parent(parent_session_id)
        .await
        .into_iter()
        .filter(|id| !initial_active.contains(id))
        .collect::<Vec<_>>();
    let mut sessions = Vec::with_capacity(ids.len());
    for id in ids {
        if let Ok(session) = super::session_store::get(&id).await {
            sessions.push(session);
        }
    }
    sessions
}

async fn active_set(parent_session_id: &str) -> BTreeSet<String> {
    super::subagent_registry::active_children_for_parent(parent_session_id)
        .await
        .into_iter()
        .collect()
}

fn reminder_message(active: &[AgentSession]) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: build_reminder_content(active),
        ..Default::default()
    }
}

pub fn build_reminder_content(active: &[AgentSession]) -> String {
    let items = active
        .iter()
        .map(format_active_subagent)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX}\n\
         <subagent_orchestration>\n\
         <instruction>Current-turn subagents are still running. Do not write a final answer yet. \
         Check active subagents, update the user briefly, then keep waiting until the required reports arrive.</instruction>\n\
         <active_subagents>\n{items}\n</active_subagents>\n\
         </subagent_orchestration>"
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

pub fn should_emit_reminder(sent: bool, last_at: Option<Instant>, now: Instant) -> bool {
    !sent || last_at.is_some_and(|last| now.duration_since(last) >= REMINDER_INTERVAL)
}

fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "subagent_orchestration_tests.rs"]
mod tests;
