use super::types_ollama::{ChatMessage, StreamResult};
use super::types_plan::AgentPlanWorkflowStatus;

const MAX_REPAIRS: usize = 2;

pub enum PlanModeDecision {
    Accept,
    Retry(&'static str),
    Fail(&'static str),
}

pub async fn evaluate(
    session_id: &str,
    result: &StreamResult,
    repair_count: usize,
) -> PlanModeDecision {
    let Ok(session) = super::session_store::get(session_id).await else {
        return PlanModeDecision::Accept;
    };
    if !session.plan_mode_enabled {
        return PlanModeDecision::Accept;
    }
    decide(session.plan_workflow_status, result, repair_count)
}

pub(crate) fn decide(
    workflow: AgentPlanWorkflowStatus,
    result: &StreamResult,
    repair_count: usize,
) -> PlanModeDecision {
    if repair_count >= MAX_REPAIRS {
        return PlanModeDecision::Fail("Plan Mode workflow could not be enforced.");
    }
    if asks_user_question(&result.content) && !has_tool(result, "ask_user_choice") {
        return PlanModeDecision::Retry(QUESTION_REPAIR);
    }
    match workflow {
        AgentPlanWorkflowStatus::AwaitingApproval => {
            if has_tool(result, "ask_user_choice") {
                PlanModeDecision::Accept
            } else {
                PlanModeDecision::Retry(APPROVAL_REPAIR)
            }
        }
        AgentPlanWorkflowStatus::Approved => {
            if has_tool(result, "exitplanmode") {
                PlanModeDecision::Accept
            } else {
                PlanModeDecision::Retry(EXIT_REPAIR)
            }
        }
        _ if result.tool_calls.is_empty() => PlanModeDecision::Retry(NEXT_ACTION_REPAIR),
        _ => PlanModeDecision::Accept,
    }
}

pub fn correction_message(content: &'static str) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content: content.to_string(),
        ..Default::default()
    }
}

fn has_tool(result: &StreamResult, name: &str) -> bool {
    result.tool_calls.iter().any(|(tool, _)| tool == name)
}

fn asks_user_question(content: &str) -> bool {
    content.contains('?') || content.contains('？')
}

const QUESTION_REPAIR: &str = "\
<plan_mode_backend_correction>
You asked the user a Plan Mode question in normal assistant text. That output is invalid and was not shown to the user.
Call ask_user_choice now with 1 to 4 concrete questions and 2 to 4 options per question. Do not answer with normal text.
</plan_mode_backend_correction>";

const APPROVAL_REPAIR: &str = "\
<plan_mode_backend_correction>
The plan is published and waiting for final approval. Call ask_user_choice now with the exact question 'Mettre en oeuvre le plan ?' and the exact options 'Mettre en oeuvre le plan', 'Continuer a planifier', 'Quitter le mode plan'. Do not do anything else first.
</plan_mode_backend_correction>";

const EXIT_REPAIR: &str = "\
<plan_mode_backend_correction>
The user approved the plan. Call exitplanmode with status approved now. After it succeeds, immediately start implementation.
</plan_mode_backend_correction>";

const NEXT_ACTION_REPAIR: &str = "\
<plan_mode_backend_correction>
Plan Mode is active. You cannot finish with normal assistant text. Call ask_user_choice if you need user input, call planmode if the plan is ready, or use read-only tools if more context is needed.
</plan_mode_backend_correction>";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_plain_text_questions() {
        assert!(asks_user_question("Which target should I use?"));
        assert!(!asks_user_question("I will inspect the project."));
    }

    #[test]
    fn finds_tool_calls_by_name() {
        let result = StreamResult {
            tool_calls: vec![("ask_user_choice".into(), serde_json::json!({}))],
            ..Default::default()
        };
        assert!(has_tool(&result, "ask_user_choice"));
        assert!(!has_tool(&result, "planmode"));
    }

    #[test]
    fn rejects_plain_text_questions_without_interactive_tool() {
        let result = StreamResult {
            content: "Which option should I use?".into(),
            ..Default::default()
        };
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::NeedsContext, &result, 0),
            PlanModeDecision::Retry(_)
        ));
    }

    #[test]
    fn requires_final_approval_after_plan_is_published() {
        let result = StreamResult::default();
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::AwaitingApproval, &result, 0),
            PlanModeDecision::Retry(_)
        ));
    }

    #[test]
    fn fails_after_too_many_repairs() {
        let result = StreamResult::default();
        assert!(matches!(
            decide(AgentPlanWorkflowStatus::NeedsContext, &result, MAX_REPAIRS),
            PlanModeDecision::Fail(_)
        ));
    }
}
