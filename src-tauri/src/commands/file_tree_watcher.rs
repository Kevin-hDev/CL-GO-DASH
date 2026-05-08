use std::sync::Mutex;

use notify::{RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, State};

use super::file_tree::validate_path;

pub struct FileTreeWatcher {
    inner: Mutex<Option<WatcherState>>,
}

struct WatcherState {
    _watcher: notify::RecommendedWatcher,
    _path: String,
}

impl FileTreeWatcher {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct FileTreeChangedPayload {
    path: String,
    kind: String,
}

#[tauri::command]
pub fn watch_project_directory(
    path: String,
    app: AppHandle,
    state: State<'_, FileTreeWatcher>,
) -> Result<(), String> {
    validate_path(&path)?;

    let canonical =
        std::fs::canonicalize(&path).map_err(|_| "Dossier introuvable".to_string())?;

    if !canonical.is_dir() {
        return Err("Dossier introuvable".into());
    }

    let canonical_str = canonical.to_string_lossy().to_string();

    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "Erreur interne".to_string())?;

    *guard = None;

    let app_handle = app.clone();
    let watched_path = canonical_str.clone();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let event = match res {
            Ok(e) => e,
            Err(_) => return,
        };

        let kind = format!("{:?}", event.kind);

        for changed in &event.paths {
            let parent = changed
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| watched_path.clone());

            let _ = app_handle.emit(
                "file-tree-changed",
                FileTreeChangedPayload {
                    path: parent,
                    kind: kind.clone(),
                },
            );
        }
    })
    .map_err(|_| "Impossible de surveiller ce dossier".to_string())?;

    watcher
        .watch(&canonical, RecursiveMode::Recursive)
        .map_err(|_| "Impossible de surveiller ce dossier".to_string())?;

    *guard = Some(WatcherState {
        _watcher: watcher,
        _path: canonical_str,
    });

    Ok(())
}

#[tauri::command]
pub fn unwatch_project_directory(state: State<'_, FileTreeWatcher>) -> Result<(), String> {
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "Erreur interne".to_string())?;
    *guard = None;
    Ok(())
}
