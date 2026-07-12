//! Cleanup au démarrage des sous-agents orphelins.
//!
//! Au démarrage, le registry des sous-agents actifs est vide (LazyLock).
//! Toute session sur disque avec `subagent_status == "running"` est donc
//! forcément le résultat d'un crash ou d'une fermeture brutale précédente.
//! On la reclasser en "interrupted" et on nettoie son worktree git associé.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};

use super::project_store;
use super::session_index;
use super::session_store::validate_session_id;
use super::subagent_status;
use super::types_session::{AgentSession, AgentSessionMeta};

const PRUNE_TIMEOUT_SECS: u64 = 3;

/// Nettoie les sous-agents orphelins détectés au démarrage.
///
/// Sans danger : le registry en mémoire étant vide au boot, toute session
/// "running" antérieure au démarrage est orpheline. Les erreurs individuelles
/// sont loggées et n'interrompent pas le cleanup global.
pub async fn cleanup_orphans(startup_cutoff: DateTime<Utc>) {
    let sessions_dir = crate::services::paths::data_dir().join("agent-sessions");
    let cleaned = match cleanup_orphans_in_dir(&sessions_dir, startup_cutoff, true).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("[startup-cleanup] cleanup sessions impossible: {e}");
            return;
        }
    };

    let pruned = prune_project_worktrees().await;

    eprintln!(
        "[startup-cleanup] {cleaned} sous-agent(s) orphelin(s) nettoyé(s), {pruned} projet(s) pruné(s)"
    );
}

pub(crate) async fn cleanup_orphans_in_dir(
    sessions_dir: &Path,
    startup_cutoff: DateTime<Utc>,
    remove_worktrees: bool,
) -> Result<usize, String> {
    let metas = session_index::rebuild_index_from(sessions_dir).await?;
    let mut cleaned = 0usize;

    for meta in metas
        .iter()
        .filter(|m| is_orphan_candidate(m, startup_cutoff))
    {
        let mut session = match read_session_from_dir(sessions_dir, &meta.id).await {
            Ok(session) => session,
            Err(e) => {
                eprintln!("[startup-cleanup] lecture session {}: {e}", meta.id);
                continue;
            }
        };
        session.subagent_status = Some(subagent_status::INTERRUPTED.to_string());

        if let Err(e) = write_session_to_dir(sessions_dir, &session).await {
            eprintln!("[startup-cleanup] mark_status interrupted {}: {e}", meta.id);
            continue;
        }
        cleaned += 1;

        if remove_worktrees {
            if let Err(e) = super::subagent_task_change::recover_and_remove_orphan(&session).await {
                eprintln!("[startup-cleanup] récupération worktree {}: {e}", meta.id);
            }
        }
    }

    if cleaned > 0 {
        let _ = session_index::rebuild_index_from(sessions_dir).await;
    }

    Ok(cleaned)
}

fn is_orphan_candidate(meta: &AgentSessionMeta, startup_cutoff: DateTime<Utc>) -> bool {
    meta.parent_session_id.is_some()
        && meta.subagent_status.as_deref() == Some(subagent_status::RUNNING)
        && meta.updated_at.unwrap_or(meta.created_at) <= startup_cutoff
}

async fn read_session_from_dir(dir: &Path, id: &str) -> Result<AgentSession, String> {
    let path = session_path(dir, id)?;
    let data = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Session introuvable: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("JSON invalide: {e}"))
}

async fn write_session_to_dir(dir: &Path, session: &AgentSession) -> Result<(), String> {
    let path = session_path(dir, &session.id)?;
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| e.to_string())?;
    let tmp = dir.join(format!(".{}.{}.tmp", session.id, uuid::Uuid::new_v4()));
    let data = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())
}

fn session_path(dir: &Path, id: &str) -> Result<PathBuf, String> {
    validate_session_id(id)?;
    Ok(dir.join(format!("{id}.json")))
}

/// Lance `git worktree prune` sur chaque projet connu, en parallèle et avec timeout.
/// Les projets inaccessibles sont ignorés silencieusement.
async fn prune_project_worktrees() -> usize {
    let projects = project_store::list().await.unwrap_or_default();
    let mut pruned = 0usize;

    for project in projects {
        let path = std::path::PathBuf::from(&project.path);
        if !path.is_dir() {
            continue;
        }
        if prune_one_project(&path).await {
            pruned += 1;
        }
    }
    pruned
}

async fn prune_one_project(path: &std::path::Path) -> bool {
    let fut = tokio::process::Command::new("git")
        .args(["-C"])
        .arg(path)
        .args(["worktree", "prune"])
        .kill_on_drop(true)
        .output();

    match tokio::time::timeout(Duration::from_secs(PRUNE_TIMEOUT_SECS), fut).await {
        Ok(Ok(output)) if output.status.success() => true,
        Ok(Ok(output)) => {
            eprintln!(
                "[startup-cleanup] git worktree prune échoué sur {}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
            false
        }
        Ok(Err(e)) => {
            eprintln!(
                "[startup-cleanup] git worktree prune erreur sur {}: {e}",
                path.display()
            );
            false
        }
        Err(_) => {
            eprintln!(
                "[startup-cleanup] git worktree prune timeout sur {}",
                path.display()
            );
            false
        }
    }
}

#[cfg(test)]
#[path = "subagent_startup_cleanup_tests.rs"]
mod tests;
