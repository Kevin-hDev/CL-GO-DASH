use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub use super::subagent_terminal_signal::SubagentTerminalKind;
use super::subagent_terminal_signal::{SubagentTerminalNotifier, SubagentTerminalState};

const MAX_PER_PARENT: usize = 4;
const MAX_TOTAL: usize = 8;

struct SubagentEntry {
    pub cancel: CancellationToken,
    pub parent_session_id: String,
    pub run_id: String,
    pub terminal_notifier: SubagentTerminalNotifier,
}

struct RegistryState {
    entries: HashMap<String, SubagentEntry>,
    run_ids: HashMap<String, String>,
    run_claims: HashMap<String, usize>,
}

static REGISTRY: LazyLock<Mutex<RegistryState>> = LazyLock::new(|| {
    Mutex::new(RegistryState {
        entries: HashMap::new(),
        run_ids: HashMap::new(),
        run_claims: HashMap::new(),
    })
});

pub async fn get_or_create_run_id(parent_id: &str) -> String {
    let mut state = REGISTRY.lock().await;
    let run_id = state
        .run_ids
        .entry(parent_id.to_string())
        .or_insert_with(|| uuid::Uuid::new_v4().to_string())
        .clone();
    *state.run_claims.entry(parent_id.to_string()).or_insert(0) += 1;
    run_id
}

pub async fn register(
    parent_id: &str,
    child_id: &str,
    cancel: CancellationToken,
) -> Result<String, String> {
    let mut state = REGISTRY.lock().await;
    if state.entries.len() >= MAX_TOTAL {
        return Err(format!("Limite de {MAX_TOTAL} sous-agents actifs atteinte"));
    }
    let parent_count = state
        .entries
        .values()
        .filter(|e| e.parent_session_id == parent_id)
        .count();
    if parent_count >= MAX_PER_PARENT {
        return Err(format!(
            "Limite de {MAX_PER_PARENT} sous-agents par session atteinte"
        ));
    }
    let run_id = state
        .run_ids
        .entry(parent_id.to_string())
        .or_insert_with(|| uuid::Uuid::new_v4().to_string())
        .clone();
    let terminal_notifier = state
        .entries
        .values()
        .find(|entry| entry.parent_session_id == parent_id)
        .map(|entry| entry.terminal_notifier.clone())
        .unwrap_or_else(super::subagent_terminal_signal::notifier);
    release_claim_locked(&mut state, parent_id);
    state.entries.insert(
        child_id.to_string(),
        SubagentEntry {
            cancel,
            parent_session_id: parent_id.to_string(),
            run_id: run_id.clone(),
            terminal_notifier,
        },
    );
    Ok(run_id)
}

pub async fn unregister(child_id: &str) {
    let mut state = REGISTRY.lock().await;
    if let Some(entry) = state.entries.remove(child_id) {
        let parent = &entry.parent_session_id;
        let remaining = state
            .entries
            .values()
            .any(|e| e.parent_session_id == *parent);
        let has_claims = state.run_claims.get(parent).copied().unwrap_or(0) > 0;
        if !remaining && !has_claims {
            state.run_ids.remove(parent);
        }
    }
}

pub async fn release_run_claim(parent_id: &str, run_id: &str) {
    let mut state = REGISTRY.lock().await;
    if state.run_ids.get(parent_id).map(String::as_str) != Some(run_id) {
        return;
    }
    release_claim_locked(&mut state, parent_id);
    let has_entries = state
        .entries
        .values()
        .any(|entry| entry.parent_session_id == parent_id);
    let has_claims = state.run_claims.get(parent_id).copied().unwrap_or(0) > 0;
    if !has_entries && !has_claims {
        state.run_ids.remove(parent_id);
    }
}

fn release_claim_locked(state: &mut RegistryState, parent_id: &str) {
    let Some(count) = state.run_claims.get_mut(parent_id) else {
        return;
    };
    if *count > 1 {
        *count -= 1;
    } else {
        state.run_claims.remove(parent_id);
    }
}

pub async fn get_run_id_for_child(child_id: &str) -> Option<String> {
    REGISTRY
        .lock()
        .await
        .entries
        .get(child_id)
        .map(|e| e.run_id.clone())
}

pub async fn active_children_for_parent(parent_id: &str) -> Vec<String> {
    REGISTRY
        .lock()
        .await
        .entries
        .iter()
        .filter(|(_, entry)| entry.parent_session_id == parent_id)
        .map(|(child_id, _)| child_id.clone())
        .collect()
}

pub async fn terminal_notifier_for_child(child_id: &str) -> Option<SubagentTerminalNotifier> {
    REGISTRY
        .lock()
        .await
        .entries
        .get(child_id)
        .map(|entry| entry.terminal_notifier.clone())
}

pub async fn subscribe_for_parent(
    parent_id: &str,
) -> Option<tokio::sync::watch::Receiver<SubagentTerminalState>> {
    REGISTRY
        .lock()
        .await
        .entries
        .values()
        .find(|entry| entry.parent_session_id == parent_id)
        .map(|entry| entry.terminal_notifier.subscribe())
}

pub async fn cancel_one(child_id: &str) -> bool {
    let state = REGISTRY.lock().await;
    if let Some(entry) = state.entries.get(child_id) {
        entry.cancel.cancel();
        true
    } else {
        false
    }
}

pub async fn cancel_all_for_parent(parent_id: &str) {
    let state = REGISTRY.lock().await;
    for entry in state.entries.values() {
        if entry.parent_session_id == parent_id {
            entry.cancel.cancel();
        }
    }
}
