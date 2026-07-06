use super::session_store::{get, validate_session_id};
use super::session_tabs_state::{
    clean_label, first_free_branch_number, normalize_tabs, validate_tabs, MAIN_TAB_ID,
    MAX_TABS_PER_SESSION,
};
use super::types_session::CloneMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

pub(super) static TABS_LOCK: Mutex<()> = Mutex::const_new(());

pub use super::session_tabs_state::{SessionTab, SessionTabs};
pub use super::session_tabs_git::{get_tab, set_clone_git_branch};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct SessionTabsFile {
    #[serde(default)]
    pub(super) sessions: HashMap<String, SessionTabs>,
}

pub(super) fn tabs_path() -> PathBuf {
    crate::services::paths::data_dir().join("session-tabs.json")
}

pub async fn list(root_session_id: &str) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let file = read_file().await?;
    let mut tabs = normalize_tabs(
        root_session_id,
        file.sessions.get(root_session_id).cloned(),
    );
    super::session_tabs_git::sync_git_branches_from_sessions(&mut tabs).await;
    Ok(tabs)
}

pub async fn save_tabs(root_session_id: &str, tabs: SessionTabs) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut normalized = normalize_tabs(root_session_id, Some(tabs));
    super::session_tabs_git::sync_git_branches_from_sessions(&mut normalized).await;
    validate_tabs(root_session_id, &normalized)?;
    file.sessions
        .insert(root_session_id.to_string(), normalized.clone());
    write_file(&file).await?;
    Ok(normalized)
}

pub async fn add_clone_tab(
    root_session_id: &str,
    clone_session_id: &str,
    parent_message_id: &str,
    mode: CloneMode,
) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    validate_session_id(clone_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let clone = get(clone_session_id).await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    if let Some(existing) = tabs.tabs.iter().find(|tab| tab.session_id == clone_session_id) {
        tabs.active_tab_id = existing.tab_id.clone();
    } else {
        push_clone_tab(
            root_session_id,
            clone_session_id,
            parent_message_id,
            mode,
            clone.git_branch,
            &mut tabs,
        )?;
    }
    file.sessions.insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    Ok(tabs)
}

pub async fn close_tab(root_session_id: &str, tab_id: &str) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    if tab_id == MAIN_TAB_ID {
        return Err("Action impossible".into());
    }
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    let removed = remove_tab(&mut tabs, tab_id)?;
    file.sessions.insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    super::session_archive::archive(&removed.session_id).await?;
    Ok(tabs)
}

pub async fn rename_tab(
    root_session_id: &str,
    tab_id: &str,
    label: &str,
) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    let label = clean_label(label)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    let Some(tab) = tabs.tabs.iter_mut().find(|tab| tab.tab_id == tab_id) else {
        return Err("Action impossible".into());
    };
    tab.label = label;
    file.sessions.insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    Ok(tabs)
}

pub async fn remove_session_from_tabs(session_id: &str) -> Result<(), String> {
    validate_session_id(session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut empty_roots = Vec::new();
    for (root_id, tabs) in &mut file.sessions {
        tabs.tabs.retain(|tab| tab.session_id != session_id);
        if tabs.tabs.is_empty() || root_id == session_id {
            empty_roots.push(root_id.clone());
        } else if !tabs.tabs.iter().any(|tab| tab.tab_id == tabs.active_tab_id) {
            tabs.active_tab_id = MAIN_TAB_ID.to_string();
        }
    }
    for root_id in empty_roots {
        file.sessions.remove(&root_id);
    }
    write_file(&file).await
}

pub async fn ensure_clone_tab(root_session_id: &str, clone_session_id: &str) -> Result<(), String> {
    let clone = get(clone_session_id).await?;
    let parent_message_id = clone
        .clone_parent_message_id
        .as_deref()
        .ok_or_else(|| "Action impossible".to_string())?;
    let mode = clone.clone_mode.clone().unwrap_or(CloneMode::Cut);
    add_clone_tab(root_session_id, clone_session_id, parent_message_id, mode).await?;
    Ok(())
}

fn push_clone_tab(
    root_session_id: &str,
    clone_session_id: &str,
    parent_message_id: &str,
    mode: CloneMode,
    git_branch: Option<String>,
    tabs: &mut SessionTabs,
) -> Result<(), String> {
    if tabs.tabs.len() >= MAX_TABS_PER_SESSION {
        return Err("Nombre maximum d'onglets atteint".into());
    }
    let branch_number = first_free_branch_number(tabs)?;
    let tab_id = format!("branch-{branch_number}");
    tabs.tabs.push(SessionTab {
        tab_id: tab_id.clone(),
        session_id: clone_session_id.to_string(),
        label: format!("Branche {branch_number}"),
        is_main: false,
        clone_parent_session_id: Some(root_session_id.to_string()),
        clone_parent_message_id: Some(parent_message_id.to_string()),
        clone_mode: Some(mode),
        git_branch,
    });
    tabs.active_tab_id = tab_id;
    Ok(())
}

fn remove_tab(tabs: &mut SessionTabs, tab_id: &str) -> Result<SessionTab, String> {
    let Some(pos) = tabs.tabs.iter().position(|tab| tab.tab_id == tab_id) else {
        return Err("Action impossible".into());
    };
    let removed = tabs.tabs.remove(pos);
    tabs.active_tab_id = MAIN_TAB_ID.to_string();
    Ok(removed)
}

pub(super) async fn read_file() -> Result<SessionTabsFile, String> {
    match tokio::fs::read_to_string(tabs_path()).await {
        Ok(data) => serde_json::from_str(&data).map_err(|_| "Fichier d'onglets invalide".into()),
        Err(_) => Ok(SessionTabsFile::default()),
    }
}

pub(super) async fn write_file(file: &SessionTabsFile) -> Result<(), String> {
    let path = tabs_path();
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    }
    let tmp = path.with_file_name(format!(".session-tabs.{}.tmp", Uuid::new_v4()));
    let data = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, data).await.map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())
}
