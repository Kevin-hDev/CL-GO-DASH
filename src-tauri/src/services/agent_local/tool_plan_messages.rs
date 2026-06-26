use super::types_plan::AgentPlanStatus;

pub const INVALID_PLAN: &str = "Invalid plan.";
pub const INVALID_STATUS: &str = "Invalid plan status.";
pub const PLAN_INACTIVE: &str = "Plan Mode is not active.";
pub const PLAN_UNAVAILABLE: &str = "Plan unavailable.";

pub fn published(title: &str) -> String {
    format!(
        "Plan '{title}' has been published. You must now call ask_user_choice for final approval \
         with the exact question 'Mettre en oeuvre le plan ?' and the exact options \
         'Mettre en oeuvre le plan', 'Continuer a planifier', 'Quitter le mode plan'. \
         Do not continue implementation before that approval."
    )
}

pub fn exited(status: AgentPlanStatus) -> &'static str {
    match status {
        AgentPlanStatus::Approved => {
            "Plan Mode exited. The plan is approved. todo_write and write tools are available again. \
             You must immediately start implementation now without waiting for another user message."
        }
        AgentPlanStatus::Rejected => "Plan Mode exited. The plan is not approved.",
        AgentPlanStatus::Cancelled => "Plan Mode cancelled.",
        _ => "Plan Mode exited.",
    }
}
