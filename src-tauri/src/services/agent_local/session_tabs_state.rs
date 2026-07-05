use super::session_store::validate_session_id;
use super::types_session::CloneMode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_TABS_PER_SESSION: usize = 3;
pub const MAIN_TAB_ID: &str = "main";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTab {
    pub tab_id: String,
    pub session_id: String,
    pub label: String,
    pub is_main: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_parent_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_mode: Option<CloneMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTabs {
    pub active_tab_id: String,
    pub tabs: Vec<SessionTab>,
}

pub fn normalize_tabs(root_session_id: &str, tabs: Option<SessionTabs>) -> SessionTabs {
    let mut normalized = tabs.unwrap_or_else(|| SessionTabs {
        active_tab_id: MAIN_TAB_ID.to_string(),
        tabs: Vec::new(),
    });
    normalized.tabs.retain(|tab| !tab.session_id.is_empty());
    normalized.tabs.truncate(MAX_TABS_PER_SESSION);
    if !normalized.tabs.iter().any(|tab| tab.is_main) {
        normalized.tabs.insert(0, main_tab(root_session_id));
    }
    for tab in &mut normalized.tabs {
        if tab.is_main {
            tab.tab_id = MAIN_TAB_ID.to_string();
            tab.session_id = root_session_id.to_string();
            tab.clone_parent_session_id = None;
            tab.clone_parent_message_id = None;
            tab.clone_mode = None;
        }
    }
    if !normalized
        .tabs
        .iter()
        .any(|tab| tab.tab_id == normalized.active_tab_id)
    {
        normalized.active_tab_id = MAIN_TAB_ID.to_string();
    }
    normalized
}

pub fn main_tab(root_session_id: &str) -> SessionTab {
    SessionTab {
        tab_id: MAIN_TAB_ID.to_string(),
        session_id: root_session_id.to_string(),
        label: "Main".to_string(),
        is_main: true,
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
    }
}

pub fn first_free_branch_number(tabs: &SessionTabs) -> Result<usize, String> {
    for number in 1..=2 {
        let tab_id = format!("branch-{number}");
        if !tabs.tabs.iter().any(|tab| tab.tab_id == tab_id) {
            return Ok(number);
        }
    }
    Err("Nombre maximum d'onglets atteint".into())
}

pub fn validate_tabs(root_session_id: &str, tabs: &SessionTabs) -> Result<(), String> {
    if tabs.tabs.len() > MAX_TABS_PER_SESSION {
        return Err("Nombre maximum d'onglets atteint".into());
    }
    let mut ids = HashSet::new();
    for tab in &tabs.tabs {
        validate_session_id(&tab.session_id)?;
        if tab.is_main && tab.session_id != root_session_id {
            return Err("Action impossible".into());
        }
        if !ids.insert(tab.tab_id.clone()) {
            return Err("Action impossible".into());
        }
    }
    Ok(())
}

pub fn clean_label(label: &str) -> Result<String, String> {
    let trimmed = label.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 80 {
        return Err("Nom invalide".into());
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn normalize_adds_main_tab() {
        let tabs = normalize_tabs(ROOT, None);
        assert_eq!(tabs.active_tab_id, MAIN_TAB_ID);
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.tabs[0].session_id, ROOT);
    }

    #[test]
    fn validate_rejects_more_than_three_tabs() {
        let mut tabs = normalize_tabs(ROOT, None);
        for number in 1..=3 {
            tabs.tabs.push(SessionTab {
                tab_id: format!("branch-{number}"),
                session_id: format!("550e8400-e29b-41d4-a716-44665544000{number}"),
                label: format!("Branch {number}"),
                is_main: false,
                clone_parent_session_id: Some(ROOT.to_string()),
                clone_parent_message_id: None,
                clone_mode: Some(CloneMode::Cut),
            });
        }
        assert!(validate_tabs(ROOT, &tabs).is_err());
    }
}
