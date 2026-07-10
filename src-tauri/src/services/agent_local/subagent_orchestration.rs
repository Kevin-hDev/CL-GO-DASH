use super::stream_events::AgentEventEmitter;
use super::types_ollama::ChatMessage;
use super::types_session::AgentSession;
use super::types_stream::{StreamEvent, StreamResult};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub const REMINDER_INTERVAL: Duration = Duration::from_secs(10 * 60);
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const REPORT_ACK_ERROR: &str = "Impossible de finaliser les rapports de sous-agents.";

pub struct ParentSubagentOrchestrator {
    parent_session_id: String,
    initial_active: BTreeSet<String>,
    last_reminder_at: Option<Instant>,
    reminder_sent: bool,
    reports_injected_since_last_request: bool,
    pending_report_ids: BTreeSet<String>,
    terminal_signal: Option<
        tokio::sync::watch::Receiver<super::subagent_terminal_signal::SubagentTerminalState>,
    >,
}

impl ParentSubagentOrchestrator {
    pub async fn new(parent_session_id: &str) -> Self {
        let initial_active = active_set(parent_session_id).await;
        Self {
            parent_session_id: parent_session_id.to_string(),
            initial_active,
            last_reminder_at: None,
            reminder_sent: false,
            reports_injected_since_last_request: false,
            pending_report_ids: BTreeSet::new(),
            terminal_signal: None,
        }
    }

    pub async fn inject_pending_reports(&mut self, messages: &mut Vec<ChatMessage>) -> bool {
        let available = super::subagent_hidden_reports::peek_reports(&self.parent_session_id).await;
        self.pending_report_ids
            .retain(|id| available.iter().any(|report| report.id == id.as_str()));
        let reports = available
            .into_iter()
            .filter(|report| !self.pending_report_ids.contains(&report.id))
            .collect::<Vec<_>>();
        if reports.is_empty() {
            return false;
        }
        self.pending_report_ids
            .extend(reports.iter().map(|report| report.id.clone()));
        while self.pending_report_ids.len() > super::subagent_hidden_reports::MAX_PENDING_REPORTS {
            self.pending_report_ids.pop_first();
        }
        super::subagent_hidden_reports::append_context(messages, &reports);
        true
    }

    pub async fn complete_model_request(&mut self, successful: bool) -> Result<(), String> {
        if !successful || self.pending_report_ids.is_empty() {
            return Ok(());
        }
        let report_ids = self.pending_report_ids.iter().cloned().collect::<Vec<_>>();
        super::subagent_hidden_reports::acknowledge_reports(&self.parent_session_id, &report_ids)
            .await
            .map_err(|_| REPORT_ACK_ERROR.to_string())?;
        self.pending_report_ids.clear();
        Ok(())
    }

    pub async fn prepare_for_model_request(&mut self, messages: &mut Vec<ChatMessage>) {
        if self.terminal_signal.is_none() {
            self.terminal_signal =
                super::subagent_registry::subscribe_for_parent(&self.parent_session_id).await;
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
        let terminal_failure = self
            .terminal_signal
            .as_ref()
            .is_some_and(|receiver| receiver.borrow().report_persistence_failed);
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
            if self
                .terminal_signal
                .as_ref()
                .is_some_and(|receiver| receiver.borrow().report_persistence_failed)
            {
                return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
            }
            if self.inject_pending_reports(messages).await {
                self.reports_injected_since_last_request = true;
                return Ok(true);
            }
            let active = self.current_turn_active().await;
            if active.is_empty() {
                return Ok(false);
            }
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

pub fn should_emit_reminder(sent: bool, last_at: Option<Instant>, now: Instant) -> bool {
    !sent || last_at.is_some_and(|last| now.duration_since(last) >= REMINDER_INTERVAL)
}

#[cfg(test)]
#[path = "subagent_orchestration_tests.rs"]
mod tests;
