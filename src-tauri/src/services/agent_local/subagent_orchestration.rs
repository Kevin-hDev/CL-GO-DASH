use super::stream_events::AgentEventEmitter;
use super::types_ollama::ChatMessage;
use super::types_session::AgentSession;
use super::types_stream::{StreamEvent, StreamResult};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub const REMINDER_INTERVAL: Duration = Duration::from_secs(10 * 60);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub struct ParentSubagentOrchestrator {
    parent_session_id: String,
    initial_active: BTreeSet<String>,
    last_reminder_at: Option<Instant>,
    reminder_sent: bool,
    reports_injected_since_last_request: bool,
    report_delivery: super::subagent_report_delivery::SubagentReportDelivery,
}

impl ParentSubagentOrchestrator {
    pub async fn new(parent_session_id: &str) -> Self {
        let initial_active = super::subagent_registry::active_children_for_parent(parent_session_id)
            .await
            .into_iter()
            .collect();
        Self {
            parent_session_id: parent_session_id.to_string(),
            initial_active,
            last_reminder_at: None,
            reminder_sent: false,
            reports_injected_since_last_request: false,
            report_delivery: super::subagent_report_delivery::SubagentReportDelivery::new(
                parent_session_id,
            ),
        }
    }

    pub async fn inject_pending_reports(&mut self, messages: &mut Vec<ChatMessage>) -> bool {
        self.report_delivery.inject_pending_reports(messages).await
    }

    pub async fn complete_model_request(
        &mut self,
        successful: bool,
        cancel: &CancellationToken,
        payload: &[ChatMessage],
    ) -> Result<(), String> {
        self.report_delivery
            .complete_model_request(successful, cancel, payload)
            .await
    }

    pub async fn prepare_for_model_request(
        &mut self,
        messages: &mut Vec<ChatMessage>,
    ) -> Result<(), String> {
        self.report_delivery.refresh_terminal_signal().await;
        if self.report_delivery.persistence_failed() {
            return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
        }
        super::subagent_orchestration_context::remove_gate_context(messages);
        let reports_injected =
            self.reports_injected_since_last_request || self.inject_pending_reports(messages).await;
        self.reports_injected_since_last_request = false;
        let active = self.current_turn_active().await;
        super::subagent_orchestration_context::replace_gate_context(
            messages,
            &active,
            reports_injected,
        );
        Ok(())
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
        let terminal_failure = self.report_delivery.persistence_failed();
        super::stream_buffer::finalize_content_phase(
            on_event,
            result,
            plan_active,
            awaiting_report || terminal_failure || !self.current_turn_active().await.is_empty(),
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
                return Err("Annulé".to_string());
            }
            self.report_delivery.refresh_terminal_signal().await;
            if self.report_delivery.persistence_failed() {
                return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
            }
            if self.inject_pending_reports(messages).await {
                self.reports_injected_since_last_request = true;
                return Ok(true);
            }
            let snapshot = super::subagent_registry::parent_snapshot(&self.parent_session_id).await;
            let active_ids = snapshot
                .active_child_ids
                .into_iter()
                .filter(|id| !self.initial_active.contains(id))
                .collect::<Vec<_>>();
            if active_ids.is_empty() {
                if snapshot
                    .terminal_state
                    .is_some_and(|state| state.report_persistence_failed)
                {
                    return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
                }
                if snapshot
                    .terminal_state
                    .is_some_and(|state| state.sequence > 0)
                {
                    if self.inject_pending_reports(messages).await {
                        self.reports_injected_since_last_request = true;
                        return Ok(true);
                    }
                    return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
                }
                return Ok(false);
            }
            let active = sessions_for_ids(active_ids).await;
            if should_emit_reminder(self.reminder_sent, self.last_reminder_at, Instant::now()) {
                super::subagent_orchestration_context::replace_gate_context(
                    messages, &active, false,
                );
                self.reminder_sent = true;
                self.last_reminder_at = Some(Instant::now());
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
    sessions_for_ids(ids).await
}

async fn sessions_for_ids(ids: Vec<String>) -> Vec<AgentSession> {
    let mut sessions = Vec::with_capacity(ids.len());
    for id in ids {
        if let Ok(session) = super::session_store::get(&id).await {
            sessions.push(session);
        }
    }
    sessions
}

pub fn should_emit_reminder(sent: bool, last_at: Option<Instant>, now: Instant) -> bool {
    !sent || last_at.is_some_and(|last| now.duration_since(last) >= REMINDER_INTERVAL)
}

#[cfg(test)]
#[path = "subagent_orchestration_tests.rs"]
mod tests;
