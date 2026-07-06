use super::session_store::validate_session_id;
use super::{session_store, session_tabs::{
    read_file, write_file, SessionTab, SessionTabs, TABS_LOCK,
}};
use super::session_tabs_state::normalize_tabs;

pub async fn get_tab(root_session_id: &str, tab_id: &str) -> Result<SessionTab, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let file = read_file().await?;
    let tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    tabs.tabs
        .into_iter()
        .find(|tab| tab.tab_id == tab_id)
        .ok_or_else(|| "Action impossible".to_string())
}

/// Lit le `main_checkpoint_branch` persisté dans `session-tabs.json` pour la session
/// principale donnée. Source de vérité pour le fallback auto lors du nettoyage de
/// branche (sans dépendre d'une valeur envoyée par le frontend).
pub async fn get_main_checkpoint_branch(root_session_id: &str) -> Result<Option<String>, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let file = read_file().await?;
    let tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    Ok(tabs.main_checkpoint_branch)
}

pub async fn set_clone_git_branch(
    root_session_id: &str,
    clone_session_id: &str,
    git_branch: Option<String>,
) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    validate_session_id(clone_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    let Some(tab) = tabs
        .tabs
        .iter_mut()
        .find(|tab| !tab.is_main && tab.session_id == clone_session_id)
    else {
        return Err("Action impossible".into());
    };
    tab.git_branch = git_branch;
    file.sessions.insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    Ok(tabs)
}

pub async fn sync_git_branches_from_sessions(tabs: &mut SessionTabs) {
    for tab in &mut tabs.tabs {
        if tab.is_main {
            tab.git_branch = None;
            continue;
        }
        if let Ok(session) = session_store::get(&tab.session_id).await {
            tab.git_branch = session.git_branch;
        }
    }
}
