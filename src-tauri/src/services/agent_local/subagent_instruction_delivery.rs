use super::types_ollama::ChatMessage;
use super::types_session::{AgentMessage, AgentSession};
use super::types_tools::ToolResult;

pub(super) const DELIVERY_ERROR: &str = "Échec interne de livraison d'une consigne sous-agent.";
pub(super) const MAX_PROMPT_SIZE: usize = 50_000;
pub(super) const MAX_QUEUED_PROMPTS: usize = 8;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum EnqueueOutcome {
    Added,
    Duplicate,
}

pub(super) fn enqueue(
    child: &mut AgentSession,
    prompt: &str,
) -> Result<EnqueueOutcome, ToolResult> {
    let normalized = normalized_prompt(prompt);
    let already_seen = child
        .subagent_queued_prompts
        .iter()
        .any(|queued| normalized_prompt(queued) == normalized)
        || child.messages.iter().any(|message| {
            message.role == "user" && normalized_prompt(&message.content) == normalized
        });
    if already_seen {
        return Ok(EnqueueOutcome::Duplicate);
    }
    if child.subagent_queued_prompts.len() >= MAX_QUEUED_PROMPTS {
        return Err(ToolResult::err("File de consignes sous-agent pleine."));
    }
    child.subagent_queued_prompts.push(prompt.to_string());
    child.subagent_status = Some(super::subagent_status::RUNNING.to_string());
    child.updated_at = Some(chrono::Utc::now());
    Ok(EnqueueOutcome::Added)
}

fn normalized_prompt(prompt: &str) -> String {
    prompt.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) async fn drain(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
) -> Result<usize, String> {
    drain_inner(session_id, messages, || async {}).await
}

async fn drain_inner<F, Fut>(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
    before_save: F,
) -> Result<usize, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let Some(expected_run) = super::subagent_registry::active_run_for_child(session_id).await else {
        return Ok(0);
    };
    if expected_run.cancelled {
        return Err(delivery_error());
    }
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut child = super::session_store::get(session_id)
        .await
        .map_err(|_| delivery_error())?;
    if child.parent_session_id.is_none() {
        return Err(delivery_error());
    }
    ensure_current_run(&child, &expected_run.run_id).await?;
    if child.subagent_queued_prompts.is_empty() {
        return Ok(0);
    }
    let prompts = child.subagent_queued_prompts.clone();
    super::session_store_messages::append_bounded(
        &mut child,
        prompts.iter().map(|prompt| agent_message(prompt)),
    );
    child.subagent_queued_prompts.clear();
    child.updated_at = Some(chrono::Utc::now());
    before_save().await;
    super::session_store::save(&child)
        .await
        .map_err(|_| delivery_error())?;
    messages.extend(prompts.iter().map(|prompt| ChatMessage {
        role: "user".to_string(),
        content: prompt.clone(),
        ..Default::default()
    }));
    Ok(prompts.len())
}

#[cfg(test)]
pub(super) async fn drain_with_before_save<F, Fut>(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
    before_save: F,
) -> Result<usize, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    drain_inner(session_id, messages, before_save).await
}

async fn ensure_current_run(child: &AgentSession, expected_run: &str) -> Result<(), String> {
    let active_run = super::subagent_registry::active_run_for_child(&child.id)
        .await
        .ok_or_else(delivery_error)?;
    if active_run.cancelled
        || active_run.run_id != expected_run
        || child.subagent_run_id.as_deref() != Some(expected_run)
    {
        return Err(delivery_error());
    }
    Ok(())
}

fn agent_message(prompt: &str) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: prompt.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    }
}

fn delivery_error() -> String {
    DELIVERY_ERROR.to_string()
}

pub(super) fn is_delivery_error(error: &str) -> bool {
    error == DELIVERY_ERROR
}
