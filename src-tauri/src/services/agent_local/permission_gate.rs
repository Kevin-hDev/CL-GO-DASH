use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamEvent;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy)]
pub enum PermissionDecision {
    Allow,
    AllowSession,
    Deny,
}

const GATED_TOOLS: &[&str] = &["bash", "write_file", "edit_file", "web_fetch"];

pub fn requires_permission(tool_name: &str) -> bool {
    GATED_TOOLS.contains(&tool_name)
}

static PENDING: LazyLock<Mutex<HashMap<String, oneshot::Sender<PermissionDecision>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn request(
    on_event: &AgentEventEmitter,
    tool_name: &str,
    arguments: &Value,
    cancel: CancellationToken,
) -> PermissionDecision {
    let id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();
    PENDING.lock().await.insert(id.clone(), tx);

    let _ = on_event.send(StreamEvent::PermissionRequest {
        id: id.clone(),
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
    });

    tokio::select! {
        res = rx => res.unwrap_or(PermissionDecision::Deny),
        _ = cancel.cancelled() => {
            PENDING.lock().await.remove(&id);
            PermissionDecision::Deny
        }
    }
}

pub async fn respond(id: &str, decision: PermissionDecision) {
    if let Some(tx) = PENDING.lock().await.remove(id) {
        let _ = tx.send(decision);
    }
}

const SESSION_ALLOW_TTL: Duration = Duration::from_secs(3600);

static ALLOWED: LazyLock<Mutex<HashMap<String, HashMap<String, Instant>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn is_allowed(session_id: &str, tool: &str) -> bool {
    let mut guard = ALLOWED.lock().await;
    let session_map = match guard.get_mut(session_id) {
        Some(m) => m,
        None => return false,
    };
    match session_map.get(tool) {
        Some(granted_at) if granted_at.elapsed() < SESSION_ALLOW_TTL => true,
        Some(_) => {
            session_map.remove(tool);
            false
        }
        None => false,
    }
}

pub async fn mark_allowed(session_id: &str, tool: &str) {
    ALLOWED
        .lock()
        .await
        .entry(session_id.to_string())
        .or_default()
        .insert(tool.to_string(), Instant::now());
}

pub async fn clear_session(session_id: &str) {
    ALLOWED.lock().await.remove(session_id);
}
