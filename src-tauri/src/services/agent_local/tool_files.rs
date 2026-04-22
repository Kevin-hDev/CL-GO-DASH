use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ToolResult;
use std::path::{Path, PathBuf};

const MAX_READ_SIZE: u64 = 20 * 1024 * 1024;
const MAX_LIST_ENTRIES: usize = 500;
pub const DEFAULT_LIMIT: usize = 2000;

fn resolve_read_path(path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let raw = if p.is_absolute() { p.to_path_buf() } else { working_dir.join(p) };
    security::validate_read_path(&raw, working_dir)
}

fn resolve_write_path(path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let raw = if p.is_absolute() { p.to_path_buf() } else { working_dir.join(p) };
    security::validate_write_path(&raw)
}

pub async fn read_file(path: &str, working_dir: &Path, offset: usize, limit: usize) -> ToolResult {
    let resolved = match resolve_read_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult { content: e, is_error: true },
    };
    match tokio::fs::metadata(&resolved).await {
        Ok(meta) if meta.len() > MAX_READ_SIZE => {
            return ToolResult {
                content: "Fichier trop volumineux (max 20MB)".into(),
                is_error: true,
            };
        }
        Err(e) => return ToolResult { content: security::sanitize_error(e), is_error: true },
        _ => {}
    }
    let raw = match tokio::fs::read_to_string(&resolved).await {
        Ok(c) => c,
        Err(e) => return ToolResult { content: security::sanitize_error(e), is_error: true },
    };
    let lines: Vec<&str> = raw.lines().collect();
    let total = lines.len();
    let start = offset.min(total);
    let end = (start + limit).min(total);
    let slice = &lines[start..end];
    let mut output = String::with_capacity(slice.len() * 80);
    for (i, line) in slice.iter().enumerate() {
        let line_num = start + i + 1;
        output.push_str(&format!("{line_num}\t{line}\n"));
    }
    let remaining = total.saturating_sub(end);
    if remaining > 0 {
        output.push_str(&format!(
            "\n[{remaining} ligne(s) restante(s) — utilise offset={end} pour la suite]"
        ));
    }
    ToolResult { content: output, is_error: false }
}

pub async fn write_file(path: &str, content: &str, working_dir: &Path) -> ToolResult {
    let resolved = match resolve_write_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult { content: e, is_error: true },
    };
    if let Some(parent) = resolved.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return ToolResult { content: security::sanitize_error(e), is_error: true };
        }
    }
    match tokio::fs::write(&resolved, content).await {
        Ok(()) => ToolResult { content: format!("Écrit: {}", resolved.display()), is_error: false },
        Err(e) => ToolResult { content: security::sanitize_error(e), is_error: true },
    }
}

pub async fn edit_file(
    path: &str,
    old_string: &str,
    new_string: &str,
    working_dir: &Path,
) -> ToolResult {
    let resolved = match resolve_write_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult { content: e, is_error: true },
    };
    let content = match tokio::fs::read_to_string(&resolved).await {
        Ok(c) => c,
        Err(e) => return ToolResult { content: security::sanitize_error(e), is_error: true },
    };
    let count = content.matches(old_string).count();
    if count == 0 {
        return ToolResult { content: "Chaîne non trouvée".into(), is_error: true };
    }
    if count > 1 {
        return ToolResult {
            content: format!("Chaîne trouvée {count} fois (doit être unique)"),
            is_error: true,
        };
    }
    let updated = content.replacen(old_string, new_string, 1);
    match tokio::fs::write(&resolved, &updated).await {
        Ok(()) => ToolResult { content: format!("Modifié: {}", resolved.display()), is_error: false },
        Err(e) => ToolResult { content: security::sanitize_error(e), is_error: true },
    }
}

pub async fn list_dir(path: &str, working_dir: &Path) -> ToolResult {
    let resolved = match resolve_read_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult { content: e, is_error: true },
    };
    let mut entries = Vec::new();
    let mut stack = vec![(resolved.clone(), 0u32)];

    while let Some((dir, depth)) = stack.pop() {
        if entries.len() >= MAX_LIST_ENTRIES {
            entries.push("... [tronqué]".to_string());
            break;
        }
        let mut read_dir = match tokio::fs::read_dir(&dir).await {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut children = Vec::new();
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            children.push(entry);
        }
        children.sort_by_key(|e| e.file_name());
        for entry in children {
            let name = entry.file_name().to_string_lossy().to_string();
            let indent = "  ".repeat(depth as usize);
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            let suffix = if is_dir { "/" } else { "" };
            entries.push(format!("{indent}{name}{suffix}"));
            if is_dir && depth < 3 {
                stack.push((entry.path(), depth + 1));
            }
        }
    }
    ToolResult { content: entries.join("\n"), is_error: false }
}
