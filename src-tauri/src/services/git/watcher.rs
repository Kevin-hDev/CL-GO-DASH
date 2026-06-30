use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: u64 = 150;

static WATCHER_ACTIVE: std::sync::Mutex<Option<Arc<AtomicBool>>> = std::sync::Mutex::new(None);

fn resolve_git_dir(repo_path: &Path) -> Option<PathBuf> {
    let dot_git = repo_path.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    if dot_git.is_file() {
        let content = std::fs::read_to_string(&dot_git).ok()?;
        let gitdir = parse_gitdir_content(&content)?;
        let resolved = if std::path::Path::new(&gitdir).is_absolute() {
            gitdir
        } else {
            repo_path.join(gitdir)
        };
        if resolved.is_dir() {
            return Some(resolved);
        }
    }
    None
}

/// Parse le contenu d'un fichier `.git` de worktree linked.
/// Format attendu : `gitdir: /chemin/vers/.git/worktrees/<name>`.
/// Retourne le chemin brut (absolu ou relatif tel qu'écrit), sans résoudre.
pub(super) fn parse_gitdir_content(content: &str) -> Option<PathBuf> {
    let gitdir = content.trim().strip_prefix("gitdir: ")?;
    if gitdir.is_empty() {
        return None;
    }
    Some(PathBuf::from(gitdir))
}

pub fn setup_git_watcher(app: AppHandle, repo_path: PathBuf) -> Result<(), String> {
    let git_dir = match resolve_git_dir(&repo_path) {
        Some(d) => d,
        None => return Ok(()),
    };

    let mut guard = WATCHER_ACTIVE
        .lock()
        .map_err(|_| "Lock error".to_string())?;

    if let Some(prev) = guard.take() {
        prev.store(true, Ordering::Relaxed);
    }

    let stop = Arc::new(AtomicBool::new(false));
    *guard = Some(Arc::clone(&stop));
    drop(guard);

    let head_path = git_dir.join("HEAD");
    let refs_path = git_dir.join("refs").join("heads");
    let packed_refs_path = git_dir.join("packed-refs");

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let all_lock = event
                    .paths
                    .iter()
                    .all(|p| p.extension().map(|e| e == "lock").unwrap_or(false));
                if all_lock {
                    return;
                }
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        let _ = tx.send(event);
                    }
                    _ => {}
                }
            }
        }) {
            Ok(w) => w,
            Err(_) => return,
        };

        if head_path.exists() {
            let _ = watcher.watch(&head_path, RecursiveMode::NonRecursive);
        }
        if refs_path.is_dir() {
            let _ = watcher.watch(&refs_path, RecursiveMode::Recursive);
        }
        if packed_refs_path.exists() {
            let _ = watcher.watch(&packed_refs_path, RecursiveMode::NonRecursive);
        }

        loop {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) => {
                    thread::sleep(Duration::from_millis(DEBOUNCE_MS));
                    while rx.try_recv().is_ok() {}
                    if !stop.load(Ordering::Relaxed) {
                        let _ = app.emit("git-branch-changed", ());
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    });

    Ok(())
}
