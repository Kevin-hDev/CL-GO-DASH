use super::tool_subagent_format::{format_child, format_meta};
use super::types_session::AgentSession;
use super::types_tools::ToolResult;
use serde_json::{json, Value};

const MAX_PROMPT_SIZE: usize = 50_000;
const MAX_QUEUED_PROMPTS: usize = 8;

pub async fn dispatch(tool_name: &str, args: &Value, parent_id: &str) -> Option<ToolResult> {
    if is_child_session(parent_id).await {
        return Some(ToolResult::err(
            "Les sous-agents ne peuvent pas piloter d'autres sous-agents.",
        ));
    }
    Some(match tool_name {
        "list_subagents" => list(parent_id).await,
        "get_subagent" => get(args, parent_id).await,
        "cancel_subagent" => cancel(args, parent_id).await,
        "message_subagent" => message(args, parent_id).await,
        "archive_subagent" => archive(args, parent_id).await,
        _ => return None,
    })
}

async fn list(parent_id: &str) -> ToolResult {
    match super::session_store::list().await {
        Ok(items) => {
            let mut rows = Vec::new();
            for item in items
                .into_iter()
                .filter(|item| item.parent_session_id.as_deref() == Some(parent_id))
            {
                rows.push(format_meta(
                    super::subagent_live_state::normalize_meta(item).await,
                ));
            }
            ToolResult::ok(if rows.is_empty() {
                "Aucun sous-agent pour cette session.".to_string()
            } else {
                rows.join("\n")
            })
        }
        Err(_) => ToolResult::err("Sous-agents indisponibles."),
    }
}

async fn get(args: &Value, parent_id: &str) -> ToolResult {
    match owned_child(args, parent_id).await {
        Ok(child) => ToolResult::ok(format_child(
            &super::subagent_live_state::normalize_session(child).await,
        )),
        Err(result) => result,
    }
}

async fn cancel(args: &Value, parent_id: &str) -> ToolResult {
    let Ok(child) = owned_child(args, parent_id).await else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    if super::subagent_registry::cancel_one(&child.id).await {
        let _ = super::session_subagents::mark_status(&child.id, super::subagent_status::CANCELLED)
            .await;
        return ToolResult::ok("Sous-agent annulé.".to_string());
    }
    ToolResult::ok("Sous-agent déjà terminé.".to_string())
}

async fn message(args: &Value, parent_id: &str) -> ToolResult {
    let prompt = match args["prompt"].as_str().map(str::trim) {
        Some(value) if !value.is_empty() && value.chars().count() <= MAX_PROMPT_SIZE => value,
        _ => return ToolResult::err("Instruction sous-agent invalide."),
    };
    let Some(child_id) = args["subagent_id"]
        .as_str()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    if super::session_store::validate_session_id(child_id).is_err() {
        return ToolResult::err("Sous-agent introuvable.");
    }
    let payload = {
        let lock = super::session_store::lock_session(child_id).await;
        let _guard = lock.lock().await;
        let Ok(mut child) = owned_child_by_id(child_id, parent_id).await else {
            return ToolResult::err("Sous-agent introuvable.");
        };
        if child_has_pending_work(&child).await {
            if let Err(result) = enqueue_prompt(&mut child, prompt) {
                return result;
            }
            if super::session_store::save(&child).await.is_err() {
                return ToolResult::err("Sous-agent indisponible.");
            }
            return ToolResult::ok("Instruction ajoutée à la file du sous-agent.".to_string());
        }
        json!({
            "subagent_id": child.id,
            "subagent_type": child.subagent_type.unwrap_or_else(|| "explorer".to_string()),
            "display_name": child.name,
            "description": child.subagent_description.unwrap_or_default(),
            "prompt": prompt
        })
    };
    super::tool_dispatcher_delegate::dispatch_delegate(&payload, parent_id).await
}

async fn archive(args: &Value, parent_id: &str) -> ToolResult {
    let Ok(child) = owned_child(args, parent_id).await else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    if let Err(result) = can_archive_child(child_has_pending_work(&child).await) {
        return result;
    }
    match super::session_store::archive(&child.id).await {
        Ok(()) => ToolResult::ok("Sous-agent archivé.".to_string()),
        Err(_) => ToolResult::err("Sous-agent indisponible."),
    }
}

fn can_archive_child(has_pending_work: bool) -> Result<(), ToolResult> {
    if has_pending_work {
        return Err(ToolResult::err("Sous-agent encore actif."));
    }
    Ok(())
}

fn enqueue_prompt(child: &mut AgentSession, prompt: &str) -> Result<(), ToolResult> {
    if child.subagent_queued_prompts.len() >= MAX_QUEUED_PROMPTS {
        return Err(ToolResult::err("File de consignes sous-agent pleine."));
    }
    child.subagent_queued_prompts.push(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.updated_at = Some(chrono::Utc::now());
    Ok(())
}

async fn owned_child(args: &Value, parent_id: &str) -> Result<AgentSession, ToolResult> {
    let Some(id) = args["subagent_id"].as_str() else {
        return Err(ToolResult::err("Sous-agent introuvable."));
    };
    owned_child_by_id(id, parent_id).await
}

async fn owned_child_by_id(id: &str, parent_id: &str) -> Result<AgentSession, ToolResult> {
    let child = super::session_store::get(id)
        .await
        .map_err(|_| ToolResult::err("Sous-agent introuvable."))?;
    if !is_owned_by_parent(&child, parent_id) {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    Ok(child)
}

fn is_owned_by_parent(child: &AgentSession, parent_id: &str) -> bool {
    child.parent_session_id.as_deref() == Some(parent_id)
}

async fn is_child_session(session_id: &str) -> bool {
    super::session_store::get(session_id)
        .await
        .map(|session| session.parent_session_id.is_some())
        .unwrap_or(false)
}

async fn child_has_pending_work(child: &AgentSession) -> bool {
    super::subagent_live_state::has_pending_work(child).await
}

#[cfg(test)]
#[path = "tool_subagent_control_tests.rs"]
mod tests;
