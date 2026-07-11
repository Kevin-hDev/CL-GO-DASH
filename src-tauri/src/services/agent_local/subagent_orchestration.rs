use super::stream_events::AgentEventEmitter;
use super::types_ollama::ChatMessage;
use super::types_stream::{StreamEvent, StreamResult};
use std::collections::BTreeSet;
use tokio_util::sync::CancellationToken;

pub struct ParentSubagentOrchestrator {
    parent_session_id: String,
    reports_injected_since_last_request: bool,
    report_delivery: super::subagent_report_delivery::SubagentReportDelivery,
}

impl ParentSubagentOrchestrator {
    pub async fn new(parent_session_id: &str) -> Self {
        Self {
            parent_session_id: parent_session_id.to_string(),
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
        super::subagent_instruction_delivery::drain(&self.parent_session_id, messages).await?;
        self.report_delivery.refresh_terminal_signal().await;
        if self.report_delivery.persistence_failed() {
            return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
        }
        super::subagent_orchestration_context::remove_gate_context(messages);
        let reports_injected =
            self.reports_injected_since_last_request || self.inject_pending_reports(messages).await;
        self.reports_injected_since_last_request = false;
        let active_ids = self.current_turn_active_ids().await;
        super::subagent_orchestration_context::replace_gate_context(
            messages,
            active_ids.len(),
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
            &BTreeSet::new(),
        )
        .await;
        let terminal_failure = self.report_delivery.persistence_failed();
        super::stream_buffer::finalize_content_phase(
            on_event,
            result,
            plan_active,
            awaiting_report || terminal_failure || !self.current_turn_active_ids().await.is_empty(),
        );
    }
    pub async fn continue_after_no_tool_turn(
        &mut self,
        on_event: &AgentEventEmitter,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
        can_request_again: bool,
    ) -> Result<bool, String> {
        if !can_request_again {
            self.ensure_no_followup_at_turn_limit().await?;
            return Ok(false);
        }
        let should_continue = super::subagent_instruction_delivery::drain(
            &self.parent_session_id,
            messages,
        )
        .await?
            > 0
            || self.after_no_tool_turn(messages, cancel).await?;
        if should_continue {
            let _ = on_event.send(StreamEvent::TurnEnd {});
        }
        Ok(should_continue)
    }

    pub async fn ensure_no_followup_at_turn_limit(&mut self) -> Result<(), String> {
        super::subagent_turn_limit::ensure(&self.parent_session_id, &mut self.report_delivery).await
    }

    pub async fn wait_after_tool_batch(
        &mut self,
        control_only: bool,
        messages: &mut Vec<ChatMessage>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        if control_only {
            let _ = self.after_no_tool_turn(messages, cancel).await?;
        }
        Ok(())
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
            let mut terminal_signal =
                super::subagent_registry::subscribe_for_parent(&self.parent_session_id).await;
            self.report_delivery.refresh_terminal_signal().await;
            if self.report_delivery.persistence_failed() {
                return Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string());
            }
            if self.inject_pending_reports(messages).await {
                self.reports_injected_since_last_request = true;
                return Ok(true);
            }
            let snapshot = super::subagent_registry::parent_snapshot(&self.parent_session_id).await;
            let active_ids = snapshot.active_child_ids;
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
            let signal = terminal_signal
                .as_mut()
                .ok_or_else(|| super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string())?;
            tokio::select! {
                _ = cancel.cancelled() => return Err("Annulé".to_string()),
                changed = signal.changed() => {
                    changed.map_err(|_| {
                        super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string()
                    })?;
                }
            }
        }
    }

    async fn current_turn_active_ids(&self) -> Vec<String> {
        current_turn_active_ids(&self.parent_session_id).await
    }
}

async fn current_turn_active_ids(parent_session_id: &str) -> Vec<String> {
    super::subagent_registry::active_children_for_parent(parent_session_id).await
}

#[cfg(test)]
#[path = "subagent_orchestration_tests.rs"]
mod tests;
