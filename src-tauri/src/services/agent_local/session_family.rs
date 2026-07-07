use super::session_store::validate_session_id;
use super::types_session::AgentSessionMeta;
use std::collections::VecDeque;

const MAX_FAMILY_SCAN: usize = 4096;

pub async fn archive_family(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let metas = super::session_index::read_index().await?;
    let ids = family_ids(&metas, id);
    for session_id in ids {
        if session_id == id {
            super::session_archive::archive(&session_id).await?;
        } else {
            let _ = super::session_archive::archive(&session_id).await;
        }
    }
    Ok(())
}

pub async fn restore_with_parent(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let metas = super::session_index::read_index().await?;
    let meta = metas
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(generic_restore_error)?;
    let ancestors = ancestor_ids(&metas, meta)?;
    for ancestor in ancestors.iter().rev() {
        super::session_archive::restore(ancestor).await?;
    }
    super::session_archive::restore(id).await?;
    if meta.clone_parent_session_id.is_some() {
        let root_id = super::clone_roots::root_id_from_metas(
            &metas,
            &meta.id,
            meta.clone_parent_session_id.as_deref(),
        )?;
        super::session_tabs::ensure_clone_tab(&root_id, id).await?;
    }
    Ok(())
}

pub async fn delete_family(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let metas = super::session_index::read_index().await?;
    let ids = family_ids(&metas, id);
    for session_id in ids.iter().rev() {
        let _ = super::session_tabs::remove_session_from_tabs(session_id).await;
        let _ = super::session_store::delete_one(session_id).await;
    }
    Ok(())
}

pub(crate) fn family_ids(metas: &[AgentSessionMeta], id: &str) -> Vec<String> {
    let mut result = vec![id.to_string()];
    let mut queue = VecDeque::from([id.to_string()]);
    while let Some(parent) = queue.pop_front() {
        if result.len() >= MAX_FAMILY_SCAN {
            break;
        }
        for meta in metas {
            if parent_id(meta) != Some(parent.as_str())
                || result.iter().any(|seen| seen == &meta.id)
            {
                continue;
            }
            result.push(meta.id.clone());
            queue.push_back(meta.id.clone());
        }
    }
    result
}

fn ancestor_ids(
    metas: &[AgentSessionMeta],
    meta: &AgentSessionMeta,
) -> Result<Vec<String>, String> {
    let mut ancestors = Vec::new();
    let mut current = meta;
    while let Some(parent) = parent_id(current) {
        if ancestors.len() >= MAX_FAMILY_SCAN {
            return Err(generic_restore_error());
        }
        let Some(parent_meta) = metas.iter().find(|entry| entry.id == parent) else {
            return Err(generic_restore_error());
        };
        ancestors.push(parent_meta.id.clone());
        current = parent_meta;
    }
    Ok(ancestors)
}

fn parent_id(meta: &AgentSessionMeta) -> Option<&str> {
    meta.clone_parent_session_id
        .as_deref()
        .or(meta.parent_session_id.as_deref())
}

fn generic_restore_error() -> String {
    "Impossible de restaurer cette session.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn meta(id: &str, parent: Option<&str>, clone_parent: Option<&str>) -> AgentSessionMeta {
        AgentSessionMeta {
            id: id.into(),
            name: id.into(),
            created_at: Utc::now(),
            updated_at: None,
            archived_at: None,
            model: "llama3".into(),
            provider: "ollama".into(),
            thinking_enabled: false,
            reasoning_mode: None,
            message_count: 0,
            is_heartbeat: false,
            is_gateway: false,
            gateway_channel_key: None,
            project_id: None,
            parent_session_id: parent.map(str::to_string),
            subagent_type: None,
            subagent_status: None,
            subagent_run_id: None,
            subagent_description: None,
            subagent_color_key: None,
            subagent_summary: None,
            clone_parent_session_id: clone_parent.map(str::to_string),
            clone_parent_message_id: None,
            clone_mode: None,
            clone_root_session_id: None,
            git_branch: None,
        }
    }

    #[test]
    fn family_ids_collects_clones_and_subagents_recursively() {
        let metas = vec![
            meta("root", None, None),
            meta("clone", None, Some("root")),
            meta("sub", Some("root"), None),
            meta("nested", Some("clone"), None),
        ];
        let ids = family_ids(&metas, "root");
        assert_eq!(ids, vec!["root", "clone", "sub", "nested"]);
    }
}
