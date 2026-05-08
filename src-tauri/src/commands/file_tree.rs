use std::path::{Component, Path};
use std::sync::Mutex;

use notify::{RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, State};

use crate::models::file_tree::FileEntry;

// ── File-tree watcher state ──────────────────────────────────────────────────

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

// ── watch / unwatch commands ─────────────────────────────────────────────────

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

    // Drop l'ancien watcher avant d'en créer un nouveau
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
            // Émettre le dossier parent du fichier modifié
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

pub const HIDDEN_ENTRIES: &[&str] = &[
    ".git",
    ".DS_Store",
    ".next",
    ".turbo",
    "__pycache__",
    "dist",
    "target",
    "build",
    ".cache",
];

const MAX_PATH_LEN: usize = 4096;
const MAX_ENTRIES: usize = 5000;

fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
        return Err("Chemin invalide".into());
    }
    if Path::new(path)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Chemin invalide".into());
    }
    Ok(())
}

fn extract_extension(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy();
    // Dotfile sans extension réelle : ".env", ".gitignore"
    if name.starts_with('.') && !name[1..].contains('.') {
        return None;
    }
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
}

#[tauri::command]
pub async fn list_directory(
    path: String,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, String> {
    validate_path(&path)?;

    let canonical = std::fs::canonicalize(&path).map_err(|_| "Dossier introuvable".to_string())?;

    if !canonical.is_dir() {
        return Err("Dossier introuvable".into());
    }

    let read_dir =
        std::fs::read_dir(&canonical).map_err(|_| "Impossible de lire ce dossier".to_string())?;

    let mut dirs: Vec<FileEntry> = Vec::new();
    let mut files: Vec<FileEntry> = Vec::new();
    let mut count = 0usize;

    for entry_result in read_dir {
        if count >= MAX_ENTRIES {
            break;
        }
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();

        if !show_hidden && HIDDEN_ENTRIES.contains(&name.as_str()) {
            continue;
        }

        let entry_path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = meta.is_dir();
        let extension = if is_dir {
            None
        } else {
            extract_extension(&entry_path)
        };

        let fe = FileEntry {
            name: name.clone(),
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
            extension,
        };

        if is_dir {
            dirs.push(fe);
        } else {
            files.push(fe);
        }
        count += 1;
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    dirs.extend(files);
    Ok(dirs)
}
