use super::clone_summary;
use super::session_store::{self, validate_session_id};
use super::session_tabs::SessionTabs;
use super::types_ollama::ChatMessage;
use super::types_session::{AgentMessage, AgentSession, CloneMode};
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const SUMMARY_MAX_TOKENS: u32 = 3072;
const MAX_ACTIVE_SUMMARIES: usize = 32;

static SUMMARY_OPS: LazyLock<Mutex<HashMap<String, CancellationToken>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize)]
pub struct CloneSessionResult {
    pub root_session_id: String,
    pub clone_session_id: String,
    pub operation_id: String,
    pub tabs: SessionTabs,
}

pub async fn clone_session(
    session_id: &str,
    message_id: &str,
    mode: CloneMode,
    custom_focus: Option<String>,
    operation_id: Option<String>,
) -> Result<CloneSessionResult, String> {
    validate_session_id(session_id)?;
    validate_simple_id(message_id, 128)?;
    let operation_id = operation_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    validate_simple_id(&operation_id, 128)?;
    let cancel = register_operation(&operation_id).await?;
    let result =
        clone_session_inner(session_id, message_id, mode, custom_focus, &operation_id, cancel)
            .await;
    finish_operation(&operation_id).await;
    result
}

pub async fn cancel_clone_summary(operation_id: &str) -> Result<(), String> {
    validate_simple_id(operation_id, 128)?;
    let map = SUMMARY_OPS.lock().await;
    if let Some(token) = map.get(operation_id) {
        token.cancel();
    }
    Ok(())
}

async fn clone_session_inner(
    session_id: &str,
    message_id: &str,
    mode: CloneMode,
    custom_focus: Option<String>,
    operation_id: &str,
    cancel: CancellationToken,
) -> Result<CloneSessionResult, String> {
    let source = session_store::get(session_id).await?;
    if source.parent_session_id.is_some() || source.clone_parent_session_id.is_some() {
        return Err("Action impossible".into());
    }
    let Some(index) = source.messages.iter().position(|message| message.id == message_id) else {
        return Err("Action impossible".into());
    };
    let suffix = source.messages[index + 1..].to_vec();
    if matches!(mode, CloneMode::Summary) && suffix.is_empty() {
        return Err("Action impossible".into());
    }
    let mut clone = build_clone(&source, message_id, mode.clone(), index);
    let clone_id = clone.id.clone();
    session_store::save(&clone).await?;
    let tabs =
        super::session_tabs::add_clone_tab(session_id, &clone_id, message_id, mode.clone())
            .await?;
    if matches!(mode, CloneMode::Summary) {
        match complete_summary(&mut clone, &suffix, custom_focus.as_deref(), cancel).await {
            Ok(()) => session_store::save(&clone).await?,
            Err(err) => {
                let _ = super::session_tabs::remove_session_from_tabs(&clone_id).await;
                let _ = session_store::delete_one(&clone_id).await;
                return Err(err);
            }
        }
    }
    Ok(CloneSessionResult {
        root_session_id: session_id.to_string(),
        clone_session_id: clone_id,
        operation_id: operation_id.to_string(),
        tabs,
    })
}

fn build_clone(
    source: &AgentSession,
    message_id: &str,
    mode: CloneMode,
    message_index: usize,
) -> AgentSession {
    let now = Utc::now();
    let mut clone = source.clone();
    clone.id = Uuid::new_v4().to_string();
    clone.name = format!("Clone - {}", source.name);
    clone.created_at = now;
    clone.updated_at = Some(now);
    clone.archived_at = None;
    clone.messages = source.messages[..=message_index].to_vec();
    clone.accumulated_tokens =
        crate::services::token_counting::estimate_agent_messages_tokens(&clone.messages);
    clone.stream_failures.clear();
    clone.diagnostic_runs.clear();
    clone.clone_parent_session_id = Some(source.id.clone());
    clone.clone_parent_message_id = Some(message_id.to_string());
    clone.clone_mode = Some(mode);
    clone.clone_summary = None;
    clone.clone_read_files.clear();
    clone.clone_modified_files.clear();
    clone.parent_session_id = None;
    clone.subagent_type = None;
    clone.subagent_worktree = None;
    clone.subagent_prompt = None;
    clone.subagent_status = None;
    clone.subagent_run_id = None;
    clone
}

async fn complete_summary(
    clone: &mut AgentSession,
    suffix: &[AgentMessage],
    custom_focus: Option<&str>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let serialized = clone_summary::serialize_messages(suffix);
    let messages = clone_summary::build_summary_messages(&serialized, custom_focus);
    let summary = collect_summary(&clone.provider, &clone.model, messages, cancel).await?;
    let summary = crate::services::compress::prompt::extract_summary(&summary);
    let (read_files, modified_files) = clone_summary::extract_traced_files(suffix);
    clone.clone_summary = Some(summary.clone());
    clone.clone_read_files = read_files;
    clone.clone_modified_files = modified_files;
    clone.messages.push(hidden_context_message(&summary));
    clone.accumulated_tokens =
        crate::services::token_counting::estimate_agent_messages_tokens(&clone.messages);
    clone.updated_at = Some(Utc::now());
    Ok(())
}

async fn collect_summary(
    provider: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    cancel: CancellationToken,
) -> Result<String, String> {
    if provider == "ollama" {
        let request = super::ollama_collect::collect_chat_with_timeout_and_limit(
            model,
            messages,
            Duration::from_secs(180),
            Some(SUMMARY_MAX_TOKENS),
        );
        return tokio::select! {
            _ = cancel.cancelled() => Err("Annulé".to_string()),
            result = request => result.map(|(content, _)| content),
        };
    }
    crate::services::llm::stream::collect_chat_silent_for_compression(
        provider,
        model,
        &messages,
        SUMMARY_MAX_TOKENS,
        cancel,
    )
    .await
    .map(|result| result.content)
}

fn hidden_context_message(summary: &str) -> AgentMessage {
    AgentMessage {
        id: Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: clone_summary::hidden_context_content(summary),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
    }
}

async fn register_operation(operation_id: &str) -> Result<CancellationToken, String> {
    let mut map = SUMMARY_OPS.lock().await;
    if map.len() >= MAX_ACTIVE_SUMMARIES {
        return Err("Trop d'opérations en cours".into());
    }
    let token = CancellationToken::new();
    map.insert(operation_id.to_string(), token.clone());
    Ok(token)
}

async fn finish_operation(operation_id: &str) {
    SUMMARY_OPS.lock().await.remove(operation_id);
}

fn validate_simple_id(id: &str, max_len: usize) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= max_len
        && id.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err("Identifiant invalide".into())
    }
}

#[cfg(test)]
#[path = "clone_session_tests.rs"]
mod tests;
