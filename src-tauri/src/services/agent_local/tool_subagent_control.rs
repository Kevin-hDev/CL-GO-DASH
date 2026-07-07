use super::types_session::{AgentSession, AgentSessionMeta};
use super::types_tools::ToolResult;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const DEFAULT_WAIT_MS: u64 = 30_000;
const MAX_WAIT_MS: u64 = 300_000;
const MAX_PROMPT_SIZE: usize = 50_000;
const MAX_QUEUED_PROMPTS: usize = 8;

pub async fn dispatch(tool_name: &str, args: &Value, parent_id: &str) -> Option<ToolResult> {
    Some(match tool_name {
        "list_subagents" => list(parent_id).await,
        "get_subagent" => get(args, parent_id).await,
        "wait_subagent" => wait(args, parent_id).await,
        "cancel_subagent" => cancel(args, parent_id).await,
        "message_subagent" => message(args, parent_id).await,
        _ => return None,
    })
}

async fn list(parent_id: &str) -> ToolResult {
    match super::session_store::list().await {
        Ok(items) => {
            let rows = items
                .into_iter()
                .filter(|item| item.parent_session_id.as_deref() == Some(parent_id))
                .map(format_meta)
                .collect::<Vec<_>>();
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
        Ok(child) => ToolResult::ok(format_child(&child)),
        Err(result) => result,
    }
}

async fn wait(args: &Value, parent_id: &str) -> ToolResult {
    let ids = match subagent_ids(args) {
        Ok(ids) => ids,
        Err(result) => return result,
    };
    let timeout = args["timeout_ms"]
        .as_u64()
        .unwrap_or(DEFAULT_WAIT_MS)
        .min(MAX_WAIT_MS);
    let started = Instant::now();
    loop {
        let mut children = Vec::with_capacity(ids.len());
        let mut any_running = false;
        for id in &ids {
            match owned_child_by_id(id, parent_id).await {
                Ok(child) => {
                    any_running |=
                        child.subagent_status.as_deref() == Some(super::subagent_status::RUNNING);
                    children.push(child);
                }
                Err(result) => return result,
            }
        }
        if !any_running || started.elapsed() >= Duration::from_millis(timeout) {
            return ToolResult::ok(format_children(&children));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
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
    let Ok(mut child) = owned_child(args, parent_id).await else {
        return ToolResult::err("Sous-agent introuvable.");
    };
    if child.subagent_status.as_deref() == Some(super::subagent_status::RUNNING) {
        if child.subagent_queued_prompts.len() >= MAX_QUEUED_PROMPTS {
            return ToolResult::err("File de consignes sous-agent pleine.");
        }
        child.subagent_queued_prompts.push(prompt.to_string());
        if super::session_store::save(&child).await.is_err() {
            return ToolResult::err("Sous-agent indisponible.");
        }
        return ToolResult::ok("Instruction ajoutée à la file du sous-agent.".to_string());
    }
    let payload = json!({
        "subagent_id": child.id,
        "subagent_type": child.subagent_type.unwrap_or_else(|| "explorer".to_string()),
        "display_name": child.name,
        "description": child.subagent_description.unwrap_or_default(),
        "prompt": prompt,
        "mode": "detach"
    });
    super::tool_dispatcher_delegate::dispatch_delegate(&payload, parent_id).await
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
    if child.parent_session_id.as_deref() != Some(parent_id) {
        return Err(ToolResult::err("Sous-agent introuvable."));
    }
    Ok(child)
}

fn subagent_ids(args: &Value) -> Result<Vec<String>, ToolResult> {
    if let Some(ids) = args["subagent_ids"].as_array() {
        let values = ids
            .iter()
            .filter_map(|value| value.as_str().map(str::trim))
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !values.is_empty() {
            return Ok(values);
        }
    }
    if let Some(id) = args["subagent_id"]
        .as_str()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        return Ok(vec![id.to_string()]);
    }
    Err(ToolResult::err("Sous-agent introuvable."))
}

fn format_meta(meta: AgentSessionMeta) -> String {
    format!(
        "- id={} name=\"{}\" type={} status={} description=\"{}\"",
        meta.id,
        text_field(&meta.name),
        meta.subagent_type.unwrap_or_else(|| "explorer".to_string()),
        meta.subagent_status
            .unwrap_or_else(|| "completed".to_string()),
        text_field(&meta.subagent_description.unwrap_or_default())
    )
}

fn format_children(children: &[AgentSession]) -> String {
    children
        .iter()
        .map(format_child)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_child(child: &AgentSession) -> String {
    format!(
        "<subagent id=\"{}\" name=\"{}\" type=\"{}\" status=\"{}\">\n<description>{}</description>\n<summary>{}</summary>\n</subagent>",
        xml_attr(&child.id),
        xml_attr(&child.name),
        xml_attr(child.subagent_type.as_deref().unwrap_or("explorer")),
        xml_attr(child.subagent_status.as_deref().unwrap_or("completed")),
        xml_text(child.subagent_description.as_deref().unwrap_or("")),
        xml_text(child.subagent_summary.as_deref().unwrap_or(""))
    )
}

fn text_field(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn xml_attr(value: &str) -> String {
    text_field(value)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
