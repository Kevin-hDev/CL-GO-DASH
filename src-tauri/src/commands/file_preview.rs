use crate::services::agent_local::security;
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const MAX_PREVIEW_SIZE: u64 = 2 * 1024 * 1024;
const MAX_PATH_LEN: usize = 4096;

#[derive(Debug, Clone, Serialize)]
pub struct EditorInfo {
    pub id: String,
    pub label: String,
}

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
pub fn detect_preview_editors() -> Result<Vec<EditorInfo>, String> {
    let mut editors = Vec::new();
    for (id, label, command) in editor_definitions() {
        if command_available(command) {
            editors.push(EditorInfo {
                id: id.to_string(),
                label: label.to_string(),
            });
        }
    }
    Ok(editors)
}

#[tauri::command]
pub fn open_preview_file(path: String, base_dir: Option<String>) -> Result<(), String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    #[cfg(target_os = "macos")]
    return spawn("open", &[resolved.as_os_str()]);
    #[cfg(target_os = "linux")]
    return spawn("xdg-open", &[resolved.as_os_str()]);
    #[cfg(target_os = "windows")]
    return spawn("explorer.exe", &[resolved.as_os_str()]);
}

#[tauri::command]
pub fn open_preview_with_editor(
    path: String,
    base_dir: Option<String>,
    editor: String,
) -> Result<(), String> {
    let resolved = resolve_preview_path(&path, base_dir.as_deref())?;
    match editor.as_str() {
        "code" => spawn("code", &[resolved.as_os_str()]),
        "zed" => spawn("zed", &[resolved.as_os_str()]),
        "sublime" => spawn(sublime_command(), &[resolved.as_os_str()]),
        "vim" => spawn("vim", &[resolved.as_os_str()]),
        "notepad" => spawn("notepad.exe", &[resolved.as_os_str()]),
        #[cfg(target_os = "macos")]
        "textedit" => spawn("open", &[std::ffi::OsStr::new("-a"), std::ffi::OsStr::new("TextEdit"), resolved.as_os_str()]),
        _ => Err("Éditeur non autorisé".into()),
    }
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

fn editor_definitions() -> Vec<(&'static str, &'static str, &'static str)> {
    let mut editors = vec![
        ("code", "Visual Studio Code", "code"),
        ("zed", "Zed", "zed"),
        ("sublime", "Sublime Text", sublime_command()),
        ("vim", "Vim", "vim"),
    ];
    #[cfg(target_os = "windows")]
    editors.push(("notepad", "Notepad", "notepad.exe"));
    #[cfg(target_os = "macos")]
    editors.push(("textedit", "TextEdit", "open"));
    editors
}

fn command_available(command: &str) -> bool {
    #[cfg(target_os = "macos")]
    if command == "open" {
        return Path::new("/System/Applications/TextEdit.app").exists()
            || Path::new("/Applications/TextEdit.app").exists();
    }

    #[cfg(target_os = "windows")]
    if command == "notepad.exe" {
        return Path::new("C:\\Windows\\notepad.exe").exists()
            || Path::new("C:\\Windows\\System32\\notepad.exe").exists();
    }

    Command::new(command)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|output| output.status.success() || !output.stdout.is_empty())
        .unwrap_or(false)
}

fn sublime_command() -> &'static str {
    if cfg!(target_os = "macos") {
        "subl"
    } else if cfg!(target_os = "windows") {
        "subl.exe"
    } else {
        "subl"
    }
}

fn spawn(command: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    Command::new(command)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|_| "Impossible d'ouvrir l'éditeur".to_string())
}
