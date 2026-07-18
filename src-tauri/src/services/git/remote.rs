use git2::{ErrorCode, PushOptions, RemoteCallbacks};
use serde::Serialize;
use std::path::Path;
use zeroize::Zeroizing;

use super::{remote_credentials, repo};

const MAX_REMOTES: usize = 32;

#[derive(Debug, Clone, Serialize)]
pub struct RemoteStatus {
    pub has_remote: bool,
    pub is_github: bool,
    pub has_upstream: bool,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PushError {
    NoRemote,
    AuthenticationRequired,
    RemoteChanged,
    InternalError,
}

pub fn status(repo_path: &Path) -> Result<RemoteStatus, String> {
    let repository = repo::open(repo_path)?;
    let has_remote = repository
        .remotes()
        .map(|items| !items.is_empty())
        .unwrap_or(false);
    let Some(branch) = current_branch(&repository).ok() else {
        return Ok(RemoteStatus {
            has_remote,
            is_github: false,
            has_upstream: false,
            ahead: 0,
            behind: 0,
        });
    };
    let name = branch_name(&branch)?;
    let is_github =
        selected_remote_url(&repository, &name).is_some_and(|url| repo::is_github_url(&url));
    let Ok(upstream) = branch.upstream() else {
        return Ok(RemoteStatus {
            has_remote,
            is_github,
            has_upstream: false,
            ahead: 0,
            behind: 0,
        });
    };
    let Some(local_oid) = branch.get().target() else {
        return Err("Référence locale invalide".to_string());
    };
    let Some(remote_oid) = upstream.get().target() else {
        return Err("Référence distante invalide".to_string());
    };
    let (ahead, behind) = repository
        .graph_ahead_behind(local_oid, remote_oid)
        .map_err(|_| "Statut distant indisponible".to_string())?;
    Ok(RemoteStatus {
        has_remote,
        is_github,
        has_upstream: true,
        ahead,
        behind,
    })
}

pub fn remote_requires_github_token(repo_path: &Path) -> bool {
    let Ok(repository) = repo::open(repo_path) else {
        return false;
    };
    let Ok(branch) = current_branch(&repository) else {
        return false;
    };
    let Ok(name) = branch_name(&branch) else {
        return false;
    };
    selected_remote_url(&repository, &name)
        .is_some_and(|url| url.starts_with("https://github.com/"))
}

pub fn push_current(
    repo_path: &Path,
    github_token: Option<Zeroizing<String>>,
) -> Result<(), PushError> {
    let repository = repo::open(repo_path).map_err(|_| PushError::InternalError)?;
    let mut branch = current_branch(&repository).map_err(|_| PushError::InternalError)?;
    let name = branch_name(&branch).map_err(|_| PushError::InternalError)?;
    let remote_name = choose_remote(&repository, &name)?;
    let mut remote = repository
        .find_remote(&remote_name)
        .map_err(|_| PushError::NoRemote)?;
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        remote_credentials::credentials(
            url,
            username,
            allowed,
            github_token.as_ref().map(|token| token.as_str()),
        )
    });
    let mut options = PushOptions::new();
    options.remote_callbacks(callbacks);
    let refspec = format!("refs/heads/{name}:refs/heads/{name}");
    remote
        .push(&[refspec.as_str()], Some(&mut options))
        .map_err(map_push_error)?;

    let oid = branch.get().target().ok_or(PushError::InternalError)?;
    repository
        .reference(
            &format!("refs/remotes/{remote_name}/{name}"),
            oid,
            true,
            "track branch published by CL-GO",
        )
        .map_err(|_| PushError::InternalError)?;
    branch
        .set_upstream(Some(&format!("{remote_name}/{name}")))
        .map_err(|_| PushError::InternalError)?;
    Ok(())
}

fn current_branch(repository: &git2::Repository) -> Result<git2::Branch<'_>, String> {
    let head = repository
        .head()
        .map_err(|_| "Branche introuvable".to_string())?;
    if !head.is_branch() {
        return Err("Branche introuvable".to_string());
    }
    Ok(git2::Branch::wrap(head))
}

fn branch_name(branch: &git2::Branch<'_>) -> Result<String, String> {
    branch
        .name()
        .ok()
        .flatten()
        .map(str::to_string)
        .ok_or_else(|| "Branche introuvable".to_string())
}

fn choose_remote(repository: &git2::Repository, branch: &str) -> Result<String, PushError> {
    if let Ok(name) = repository
        .config()
        .and_then(|config| config.get_string(&format!("branch.{branch}.remote")))
    {
        if name != "." && repository.find_remote(&name).is_ok() {
            return Ok(name);
        }
    }
    if repository.find_remote("origin").is_ok() {
        return Ok("origin".to_string());
    }
    repository
        .remotes()
        .ok()
        .and_then(|names| {
            names
                .iter()
                .filter_map(|name| name.ok().flatten())
                .take(MAX_REMOTES)
                .next()
                .map(|name| name.to_string())
        })
        .ok_or(PushError::NoRemote)
}

fn selected_remote_url(repository: &git2::Repository, branch: &str) -> Option<String> {
    let remote_name = choose_remote(repository, branch).ok()?;
    let remote = repository.find_remote(&remote_name).ok()?;
    remote
        .pushurl()
        .ok()
        .flatten()
        .or_else(|| remote.url().ok())
        .map(str::to_string)
}

fn map_push_error(error: git2::Error) -> PushError {
    if error.code() == ErrorCode::NotFastForward {
        return PushError::RemoteChanged;
    }
    let message = error.message().to_ascii_lowercase();
    if message.contains("non-fast-forward") || message.contains("rejected") {
        PushError::RemoteChanged
    } else if message.contains("auth") || message.contains("credential") {
        PushError::AuthenticationRequired
    } else {
        PushError::InternalError
    }
}
