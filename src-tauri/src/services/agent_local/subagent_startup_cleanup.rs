//! Cleanup au démarrage des sous-agents orphelins.
//!
//! Au démarrage, le registry des sous-agents actifs est vide (LazyLock).
//! Toute session sur disque avec `subagent_status == "running"` est donc
//! forcément le résultat d'un crash ou d'une fermeture brutale précédente.
//! On la reclasser en "interrupted" et on nettoie son worktree git associé.

use std::time::Duration;

use super::project_store;
use super::session_index;
use super::session_store;
use super::session_subagents;
use super::subagent_worktree;

const PRUNE_TIMEOUT_SECS: u64 = 3;

/// Nettoie les sous-agents orphelins détectés au démarrage.
///
/// Sans danger : le registry en mémoire étant vide au boot, toute session
/// "running" est nécessairement orpheline. Les erreurs individuelles sont
/// loggées et n'interrompent pas le cleanup global.
pub async fn cleanup_orphans() {
    let metas = match session_index::read_index().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[startup-cleanup] lecture index impossible: {e}");
            return;
        }
    };

    let mut cleaned = 0usize;
    for meta in metas
        .iter()
        .filter(|m| {
            m.parent_session_id.is_some() && m.subagent_status.as_deref() == Some("running")
        })
    {
        if let Err(e) = session_subagents::mark_status(&meta.id, "interrupted").await {
            eprintln!("[startup-cleanup] mark_status interrupted {}: {e}", meta.id);
            continue;
        }
        cleaned += 1;

        if let Ok(session) = session_store::get(&meta.id).await {
            if let Some(worktree) = &session.subagent_worktree {
                if let Err(e) = subagent_worktree::remove(worktree).await {
                    eprintln!(
                        "[startup-cleanup] suppression worktree {} (session {}): {e}",
                        worktree, meta.id
                    );
                }
            }
        }
    }

    let pruned = prune_project_worktrees().await;

    eprintln!(
        "[startup-cleanup] {cleaned} sous-agent(s) orphelin(s) nettoyé(s), {pruned} projet(s) pruné(s)"
    );
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
