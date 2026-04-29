use crate::services::agent_local::security;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use super::file_preview_editors::{detect_editors_for_extension, DetectedEditor};

const MAX_PREVIEW_SIZE: u64 = 2 * 1024 * 1024;
const MAX_PATH_LEN: usize = 4096;

#[tauri::command]
pub async fn read_file_preview(path: String, base_dir: Option<String>) -> Result<String, String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    let metadata = tokio::fs::metadata(&resolved)
        .await
        .map_err(security::sanitize_error)?;
    if !metadata.is_file() {
        return Err("Fichier non supporté".into());
    }
    if metadata.len() > MAX_PREVIEW_SIZE {
        return Err("Fichier trop volumineux pour l'aperçu".into());
    }
    let bytes = tokio::fs::read(&resolved)
        .await
        .map_err(security::sanitize_error)?;
    if bytes.contains(&0) {
        return Err("Fichier non supporté".into());
    }
    String::from_utf8(bytes).map_err(|_| "Fichier non supporté".to_string())
}

#[tauri::command]
pub fn detect_editors_for_file(
    path: String,
    base_dir: Option<String>,
) -> Result<Vec<DetectedEditor>, String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    Ok(detect_editors_for_extension(&resolved))
}

#[tauri::command]
pub fn open_preview_file(path: String, base_dir: Option<String>) -> Result<(), String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    #[cfg(target_os = "macos")]
    return spawn_cmd("open", &[resolved.as_os_str()]);
    #[cfg(target_os = "linux")]
    return spawn_cmd("xdg-open", &[resolved.as_os_str()]);
    #[cfg(target_os = "windows")]
    return spawn_cmd("explorer.exe", &[resolved.as_os_str()]);
}

#[tauri::command]
pub fn open_preview_with_editor(
    path: String,
    base_dir: Option<String>,
    editor_path: String,
) -> Result<(), String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;

    #[cfg(target_os = "macos")]
    return spawn_cmd("open", &[
        std::ffi::OsStr::new("-a"),
        std::ffi::OsStr::new(&editor_path),
        resolved.as_os_str(),
    ]);

    #[cfg(target_os = "windows")]
    return spawn_cmd(&editor_path, &[resolved.as_os_str()]);

    #[cfg(target_os = "linux")]
    return spawn_cmd("gtk-launch", &[
        std::ffi::OsStr::new(&editor_path),
        resolved.as_os_str(),
    ]);
}

fn resolve_preview_path(path: &str, base_dir: Option<&str>) -> Result<PathBuf, String> {
    validate_path_text(path)?;
    let raw_path = Path::new(path);
    let joined = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        let base = base_dir.ok_or("Chemin invalide")?;
        validate_path_text(base)?;
        PathBuf::from(base).join(raw_path)
    };
    let working_dir = base_dir.map(PathBuf::from).unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });
    security::validate_read_path(&joined, &working_dir)
}

fn validate_path_text(path: &str) -> Result<(), String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
        return Err("Chemin invalide".into());
    }
    if Path::new(path).components().any(|part| matches!(part, Component::ParentDir)) {
        return Err("Chemin invalide".into());
    }
    Ok(())
}

fn spawn_cmd(command: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    Command::new(command)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|_| "Impossible d'ouvrir l'éditeur".to_string())
}
