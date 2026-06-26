use super::stream_events::AgentEventEmitter;
use super::types_interactive::AgentInteractiveAnswer;
use super::types_plan::{
    AgentPlanApprovalDecision, AgentPlanStatus, AgentPlanWorkflowStatus,
};
use super::types_session::AgentSession;
use super::types_stream::StreamEvent;

pub async fn record_answers(
    session_id: &str,
    answers: &[AgentInteractiveAnswer],
    on_event: &AgentEventEmitter,
) -> Result<Option<&'static str>, String> {
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(session_id).await?;
    if !session.plan_mode_enabled || session.plan_workflow_status != AgentPlanWorkflowStatus::AwaitingApproval {
        return Ok(None);
    }
    match classify_answers(answers) {
        Some(AgentPlanApprovalDecision::Implement) => {
            session.plan_workflow_status = AgentPlanWorkflowStatus::Approved;
            session.plan_approval_decision = Some(AgentPlanApprovalDecision::Implement);
            super::session_store::save(&session).await?;
            Ok(Some("implement"))
        }
        Some(AgentPlanApprovalDecision::ContinuePlanning) => {
            mark_active_run(&mut session, AgentPlanStatus::Rejected);
            session.plan_workflow_status = AgentPlanWorkflowStatus::CollectingQuestions;
            session.plan_approval_decision = Some(AgentPlanApprovalDecision::ContinuePlanning);
            session.active_plan_id = None;
            super::session_store::save(&session).await?;
            let _ = on_event.send(StreamEvent::PlanPreviewUpdated { plan: None });
            Ok(Some("continue_planning"))
        }
        Some(AgentPlanApprovalDecision::QuitPlan) => {
            session.plan_workflow_status = AgentPlanWorkflowStatus::Rejected;
            session.plan_approval_decision = Some(AgentPlanApprovalDecision::QuitPlan);
            super::session_store::save(&session).await?;
            Ok(Some("quit_plan"))
        }
        None => Ok(Some("approval_unrecognized")),
    }
}

pub fn validate_exit(session: &AgentSession, status: AgentPlanStatus) -> Result<(), String> {
    match status {
        AgentPlanStatus::Approved => {
            if session.plan_workflow_status == AgentPlanWorkflowStatus::Approved
                && session.plan_approval_decision == Some(AgentPlanApprovalDecision::Implement)
            {
                Ok(())
            } else {
                Err("Validation utilisateur requise avant sortie du Mode plan.".to_string())
            }
        }
        AgentPlanStatus::Rejected => {
            if session.plan_approval_decision == Some(AgentPlanApprovalDecision::QuitPlan) {
                Ok(())
            } else {
                Err("Choix utilisateur requis avant de quitter le Mode plan.".to_string())
            }
        }
        AgentPlanStatus::Cancelled => Ok(()),
        _ => Err("Statut de plan invalide.".to_string()),
    }
}

pub(crate) fn classify_answers(
    answers: &[AgentInteractiveAnswer],
) -> Option<AgentPlanApprovalDecision> {
    let text = answers
        .iter()
        .flat_map(|answer| answer.selected_labels.iter())
        .map(|label| normalize(label))
        .collect::<Vec<_>>()
        .join(" ");
    if text.contains("mettre") && text.contains("oeuvre") && text.contains("plan") {
        return Some(AgentPlanApprovalDecision::Implement);
    }
    if text.contains("continuer") && text.contains("planifier") {
        return Some(AgentPlanApprovalDecision::ContinuePlanning);
    }
    if text.contains("quitter") && text.contains("mode") && text.contains("plan") {
        return Some(AgentPlanApprovalDecision::QuitPlan);
    }
    None
}

fn mark_active_run(session: &mut AgentSession, status: AgentPlanStatus) {
    if let Some(active_id) = session.active_plan_id.clone() {
        if let Some(run) = session.plan_runs.iter_mut().find(|run| run.id == active_id) {
            run.status = status;
            run.updated_at = chrono::Utc::now();
        }
    }
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .replace('œ', "oe")
        .replace(['é', 'è', 'ê'], "e")
        .replace('à', "a")
        .replace('ù', "u")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_session::AgentMessage;
    use chrono::Utc;

    fn answer(label: &str) -> AgentInteractiveAnswer {
        AgentInteractiveAnswer {
            question_index: 0,
            selected_labels: vec![label.to_string()],
            custom_answer: None,
        }
    }

    fn session(
        workflow: AgentPlanWorkflowStatus,
        decision: Option<AgentPlanApprovalDecision>,
    ) -> AgentSession {
        AgentSession {
            id: "abc-123".into(),
            name: "Test".into(),
            created_at: Utc::now(),
            model: "llama3".into(),
            provider: "ollama".into(),
            thinking_enabled: false,
            reasoning_mode: None,
            accumulated_tokens: 0,
            messages: Vec::<AgentMessage>::new(),
            todos: vec![],
            todo_runs: vec![],
            active_todo_run_id: None,
            stream_failures: vec![],
            diagnostic_runs: vec![],
            plan_mode_enabled: true,
            plan_runs: vec![],
            active_plan_id: None,
            plan_workflow_status: workflow,
            plan_approval_decision: decision,
            is_heartbeat: false,
            is_gateway: false,
            gateway_channel_key: None,
            project_id: None,
            working_dir: String::new(),
            parent_session_id: None,
            subagent_type: None,
            subagent_worktree: None,
            subagent_prompt: None,
            subagent_status: None,
            subagent_run_id: None,
        }
    }

    #[test]
    fn classifies_plan_approval_choices() {
        assert_eq!(
            classify_answers(&[answer("Mettre en oeuvre le plan")]),
            Some(AgentPlanApprovalDecision::Implement)
        );
        assert_eq!(
            classify_answers(&[answer("Continuer a planifier")]),
            Some(AgentPlanApprovalDecision::ContinuePlanning)
        );
        assert_eq!(
            classify_answers(&[answer("Quitter le mode plan")]),
            Some(AgentPlanApprovalDecision::QuitPlan)
        );
    }

    #[test]
    fn approved_exit_requires_user_approval() {
        let blocked = session(AgentPlanWorkflowStatus::AwaitingApproval, None);
        assert!(validate_exit(&blocked, AgentPlanStatus::Approved).is_err());

        let allowed = session(
            AgentPlanWorkflowStatus::Approved,
            Some(AgentPlanApprovalDecision::Implement),
        );
        assert!(validate_exit(&allowed, AgentPlanStatus::Approved).is_ok());
    }
}
