use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: u64 = 150;

pub fn setup_git_watcher(app: AppHandle, repo_path: PathBuf) -> Result<(), String> {
    let git_dir = repo_path.join(".git");
    if !git_dir.is_dir() {
        return Ok(());
    }

    let head_path = git_dir.join("HEAD");
    let refs_path = git_dir.join("refs").join("heads");

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let dominated_by_lock = event.paths.iter().all(|p| {
                    p.extension().map(|e| e == "lock").unwrap_or(false)
                });
                if dominated_by_lock {
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
            if let Err(e) = watcher.watch(&head_path, RecursiveMode::NonRecursive) {
                eprintln!("Watch HEAD: {e}");
            }
        }

        if refs_path.is_dir() {
            if let Err(e) = watcher.watch(&refs_path, RecursiveMode::Recursive) {
                eprintln!("Watch refs/heads: {e}");
            }
        }

        loop {
            if rx.recv().is_ok() {
                thread::sleep(Duration::from_millis(DEBOUNCE_MS));
                while rx.try_recv().is_ok() {}
                let _ = app.emit("git-branch-changed", ());
            }
        }
    });

    Ok(())
}
