use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use uuid::Uuid;

static SESSION_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-f0-9\-]+$").unwrap());

pub fn validate_session_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Identifiant de session invalide".into());
    }
    if !SESSION_ID_REGEX.is_match(id) {
        return Err("Identifiant de session invalide".into());
    }
    Ok(())
}

static SESSION_LOCKS: LazyLock<Mutex<HashMap<String, std::sync::Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) async fn lock_session(id: &str) -> std::sync::Arc<Mutex<()>> {
    let mut map = SESSION_LOCKS.lock().await;
    map.entry(id.to_string())
        .or_insert_with(|| std::sync::Arc::new(Mutex::new(())))
        .clone()
}

pub async fn remove_session_lock(id: &str) {
    SESSION_LOCKS.lock().await.remove(id);
}

fn sessions_dir() -> PathBuf {
    crate::services::paths::data_dir().join("agent-sessions")
}

pub async fn create_with_flags(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
) -> Result<AgentSession, String> {
    create_full(name, model, provider, is_heartbeat, None).await
}

pub async fn create_gateway(
    name: &str,
    model: &str,
    provider: &str,
    gateway_channel_key: String,
) -> Result<AgentSession, String> {
    let mut session = create_full(name, model, provider, false, None).await?;
    session.is_gateway = true;
    session.gateway_channel_key = Some(gateway_channel_key);
    save(&session).await?;
    Ok(session)
}

pub async fn create_full(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    let reasoning_mode = crate::services::reasoning::default_mode(provider, model);
    let now = Utc::now();
    let session = AgentSession {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: now,
        updated_at: Some(now),
        archived_at: None,
        model: model.to_string(),
        provider: provider.to_string(),
        thinking_enabled: crate::services::reasoning::enabled(reasoning_mode.as_deref(), false),
        reasoning_mode,
        accumulated_tokens: 0,
        messages: Vec::new(),
        todos: Vec::new(),
        todo_neglect_count: 0,
        todo_runs: Vec::new(),
        active_todo_run_id: None,
        stream_failures: Vec::new(),
        diagnostic_runs: Vec::new(),
        plan_mode_enabled: false,
        plan_runs: Vec::new(),
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        plan_approval_decision: None,
        is_heartbeat,
        is_gateway: false,
        gateway_channel_key: None,
        project_id,
        working_dir: String::new(),
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        git_branch: None,
    };
    save(&session).await?;
    Ok(session)
}

/// Cherche la conversation heartbeat existante pour un couple (provider, model).
/// Retourne la plus récente si plusieurs existent, `None` sinon.
pub async fn find_heartbeat_session(provider: &str, model: &str) -> Result<Option<String>, String> {
    let metas = crate::services::agent_local::session_index::read_index().await?;
    let best = metas
        .iter()
        .filter(|m| super::session_archive::is_active(m) && m.is_heartbeat && m.provider == provider && m.model == model)
        .max_by_key(|m| super::session_archive::activity_at(m))
        .map(|m| m.id.clone());
    Ok(best)
}

pub async fn get(id: &str) -> Result<AgentSession, String> {
    validate_session_id(id)?;
    let path = sessions_dir().join(format!("{id}.json"));
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Session introuvable: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("JSON invalide: {e}"))
}

pub async fn list() -> Result<Vec<AgentSessionMeta>, String> {
    let mut metas = crate::services::agent_local::session_index::read_index().await?;
    metas.retain(super::session_archive::is_active);
    super::session_archive::sort_recent_first(&mut metas);
    Ok(metas)
}

pub async fn save(session: &AgentSession) -> Result<(), String> {
    validate_session_id(&session.id)?;
    let dir = sessions_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", session.id));
    let tmp = dir.join(format!(".{}.{}.tmp", session.id, Uuid::new_v4()));
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())?;
    let meta = crate::services::agent_local::session_index::meta_from_session(session);
    let _ = crate::services::agent_local::session_index::upsert_entry(meta).await;
    Ok(())
}

const MAX_MESSAGES_PER_SESSION: usize = 2000;

pub async fn add_messages(
    id: &str,
    mut new_messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
    tokens: u32,
) -> Result<(), String> {
    validate_session_id(id)?;
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    let has_user_message = new_messages.iter().any(|m| m.role == "user");
    let todo_housekeeping =
        super::session_store_todos::apply_user_turn(&mut session, has_user_message);
    if tokens > 0 {
        if let Some(last) = new_messages.last_mut() {
            last.tokens = tokens;
        }
    }
    session.messages.extend(new_messages);
    session.updated_at = Some(Utc::now());
    if session.messages.len() > MAX_MESSAGES_PER_SESSION {
        let excess = session.messages.len() - MAX_MESSAGES_PER_SESSION;
        session.messages.drain(..excess);
    }
    session.accumulated_tokens =
        crate::services::token_counting::estimate_agent_messages_tokens(&session.messages);
    let result = save(&session).await;
    if result.is_ok() && todo_housekeeping.should_emit_empty_update {
        super::tool_todo::emit_update(id, Vec::new());
    }
    result
}

pub async fn rename(id: &str, name: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let mut session = get(id).await?;
    session.name = name.to_string();
    save(&session).await
}

pub(crate) async fn delete_one(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let path = sessions_dir().join(format!("{id}.json"));
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| format!("Erreur suppression: {e}"))?;
    let _ = crate::services::agent_local::session_index::remove_entry(id).await;
    // Nettoie aussi le WriteGuard persistant de la session.
    crate::services::agent_local::write_guard_registry::remove(id);
    Ok(())
}

pub async fn delete(id: &str) -> Result<(), String> {
    super::session_family::delete_family(id).await
}

pub async fn archive(id: &str) -> Result<(), String> {
    super::session_family::archive_family(id).await
}

pub async fn restore(id: &str) -> Result<(), String> {
    super::session_family::restore_with_parent(id).await
}

pub use super::session_archive::list_archived;
pub use super::session_ops::{clear_project_id, export_markdown, truncate_and_replace};
pub use super::session_store_updates::{update_model, update_reasoning, update_working_dir};

#[path = "session_store_tests.rs"]
#[cfg(test)]
mod tests;
