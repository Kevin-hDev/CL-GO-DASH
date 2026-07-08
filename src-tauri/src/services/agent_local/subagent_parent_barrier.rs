use super::types_ollama::ChatMessage;
use super::types_session::AgentSession;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub const REMINDER_INTERVAL: Duration = Duration::from_secs(10 * 60);
const POLL_INTERVAL: Duration = Duration::from_secs(1);
pub const SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX: &str = "Subagent orchestration context:";

pub enum BarrierAction {
    Continue,
    Finish,
}

pub struct ParentSubagentBarrier {
    parent_session_id: String,
    initial_active: BTreeSet<String>,
    last_reminder_at: Option<Instant>,
    reminder_sent: bool,
}

impl ParentSubagentBarrier {
    pub async fn new(parent_session_id: &str) -> Self {
        Self {
            parent_session_id: parent_session_id.to_string(),
            initial_active: active_set(parent_session_id).await,
            last_reminder_at: None,
            reminder_sent: false,
        }
    }

    pub async fn after_no_tool_turn(
        &mut self,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
    ) -> Result<BarrierAction, String> {
        loop {
            if cancel.is_cancelled() {
                return Err("Annulé".to_string());
            }
            if append_hidden_reports(&self.parent_session_id, messages).await {
                return Ok(BarrierAction::Continue);
            }
            let active = self.current_turn_active().await;
            if active.is_empty() {
                return Ok(BarrierAction::Finish);
            }
            if should_emit_reminder(self.reminder_sent, self.last_reminder_at, Instant::now()) {
                messages.push(reminder_message(&active));
                self.reminder_sent = true;
                self.last_reminder_at = Some(Instant::now());
                return Ok(BarrierAction::Continue);
            }
            tokio::select! {
                _ = cancel.cancelled() => return Err("Annulé".to_string()),
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
            }
        }
    }

    async fn current_turn_active(&self) -> Vec<AgentSession> {
        let ids = super::subagent_registry::active_children_for_parent(&self.parent_session_id)
            .await
            .into_iter()
            .filter(|id| !self.initial_active.contains(id))
            .collect::<Vec<_>>();
        let mut sessions = Vec::with_capacity(ids.len());
        for id in ids {
            if let Ok(session) = super::session_store::get(&id).await {
                sessions.push(session);
            }
        }
        sessions
    }
}

async fn active_set(parent_session_id: &str) -> BTreeSet<String> {
    super::subagent_registry::active_children_for_parent(parent_session_id)
        .await
        .into_iter()
        .collect()
}

async fn append_hidden_reports(parent_session_id: &str, messages: &mut Vec<ChatMessage>) -> bool {
    let reports = super::subagent_hidden_reports::take_for_context(parent_session_id).await;
    if reports.is_empty() {
        return false;
    }
    messages.extend(reports);
    true
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
         Check their state with list_subagents/get_subagent/wait_subagent when useful, share only a brief progress update with the user, then keep waiting until the required reports arrive.</instruction>\n\
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
#[path = "subagent_parent_barrier_tests.rs"]
mod tests;
