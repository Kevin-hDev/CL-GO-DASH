use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const MAX_PER_PARENT: usize = 4;
const MAX_TOTAL: usize = 8;

struct SubagentEntry {
    pub cancel: CancellationToken,
    pub parent_session_id: String,
    pub run_id: String,
}

struct RegistryState {
    entries: HashMap<String, SubagentEntry>,
    run_ids: HashMap<String, String>,
}

static REGISTRY: LazyLock<Mutex<RegistryState>> = LazyLock::new(|| {
    Mutex::new(RegistryState {
        entries: HashMap::new(),
        run_ids: HashMap::new(),
    })
});

pub async fn get_or_create_run_id(parent_id: &str) -> String {
    let mut state = REGISTRY.lock().await;
    state
        .run_ids
        .entry(parent_id.to_string())
        .or_insert_with(|| uuid::Uuid::new_v4().to_string())
        .clone()
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
    state.entries.insert(
        child_id.to_string(),
        SubagentEntry {
            cancel,
            parent_session_id: parent_id.to_string(),
            run_id: run_id.clone(),
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
        if !remaining {
            state.run_ids.remove(parent);
        }
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
