use super::subagent_instruction_delivery::{EnqueueOutcome, MAX_PROMPT_SIZE};
use super::types_session::AgentSession;
use super::types_tools::ToolResult;
use serde_json::{json, Value};

pub(super) async fn run(args: &Value, parent_id: &str) -> ToolResult {
    let Some(prompt) = valid_prompt(args) else {
        return ToolResult::err("Instruction sous-agent invalide.");
    };
    let Some(child_id) = valid_child_id(args) else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    let payload = {
        let lock = super::session_store::lock_session(child_id).await;
        let _guard = lock.lock().await;
        let Ok(mut child) = super::tool_subagent_control::owned_child_by_id(child_id, parent_id).await
        else {
            return ToolResult::err("Sous-agent introuvable.");
        };
        let active_run = super::subagent_registry::active_run_for_child(&child.id).await;
        let same_run = active_run
            .as_ref()
            .is_some_and(|run| !run.cancelled && child.subagent_run_id.as_deref() == Some(&run.run_id));
        if same_run {
            return enqueue_live(&mut child, prompt).await;
        }
        if active_run.is_some()
            || child.subagent_status.as_deref() != Some(super::subagent_status::COMPLETED)
        {
            return redeploy_required();
        }
        build_resume_payload(&child, prompt)
    };
    super::tool_dispatcher_delegate::dispatch_delegate(&payload, parent_id).await
}

fn valid_prompt(args: &Value) -> Option<&str> {
    args["prompt"].as_str().filter(|value| {
        !value.trim().is_empty() && value.chars().count() <= MAX_PROMPT_SIZE
    })
}

fn valid_child_id(args: &Value) -> Option<&str> {
    let id = args["subagent_id"].as_str()?.trim();
    (!id.is_empty() && super::session_store::validate_session_id(id).is_ok()).then_some(id)
}

async fn enqueue_live(child: &mut AgentSession, prompt: &str) -> ToolResult {
    match enqueue_prompt(child, prompt) {
        Ok(EnqueueOutcome::Duplicate) => ToolResult::ok(
            "Instruction déjà présente ou livrée; aucun doublon ajouté.".to_string(),
        ),
        Ok(EnqueueOutcome::Added) => match super::session_store::save(child).await {
            Ok(()) => ToolResult::ok("Instruction ajoutée à la file du sous-agent.".to_string()),
            Err(_) => ToolResult::err("Sous-agent indisponible."),
        },
        Err(result) => result,
    }
}

fn redeploy_required() -> ToolResult {
    ToolResult::err(
        "Sous-agent terminé. Utilisez delegate_task avec subagent_id pour le redéployer.",
    )
}

pub(super) fn build_resume_payload(child: &AgentSession, prompt: &str) -> Value {
    json!({
        "subagent_id": child.id,
        "subagent_type": child.subagent_type.as_deref().unwrap_or("explorer"),
        "display_name": child.name,
        "description": child.subagent_description.as_deref().unwrap_or(""),
        "prompt": prompt,
    })
}

pub(super) fn enqueue_prompt(
    child: &mut AgentSession,
    prompt: &str,
) -> Result<EnqueueOutcome, ToolResult> {
    super::subagent_instruction_delivery::enqueue(child, prompt)
}
