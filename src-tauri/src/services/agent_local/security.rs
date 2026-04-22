use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const DESTRUCTIVE_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf *",
    "sudo rm",
    "chmod 777",
    "dd if=",
    "mkfs.",
    "> /dev/sd",
    "fdisk",
    "shutdown",
    "reboot",
    "init 0",
    "init 6",
    ":(){:|:&};:",
    "del /f /s /q",
    "rd /s /q",
    "format c:",
    "format d:",
];

static S7_EVAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"eval\s+"?\$"#).unwrap());

pub fn allowed_write_roots() -> Vec<PathBuf> {
    let mut raw = Vec::with_capacity(4);
    raw.push(crate::services::paths::data_dir());
    if let Some(home) = dirs::home_dir() {
        raw.push(home.join(".ollama"));
        raw.push(home.join("Projects"));
    }
    raw.push(std::env::temp_dir());
    raw.into_iter()
        .map(|p| p.canonicalize().unwrap_or(p))
        .collect()
}

pub fn check_destructive_command(cmd: &str) -> Result<(), String> {
    for pattern in DESTRUCTIVE_PATTERNS {
        if cmd.contains(pattern) {
            return Err(format!(
                "Commande bloquée : pattern dangereux « {pattern} »"
            ));
        }
    }
    if S7_EVAL_REGEX.is_match(cmd) {
        return Err(
            "Commande bloquée : eval avec expansion de variable (utiliser une liste d'arguments)"
                .into(),
        );
    }
    Ok(())
}

pub fn allowed_read_roots() -> Vec<PathBuf> {
    let mut raw = Vec::with_capacity(3);
    if let Some(home) = dirs::home_dir() {
        raw.push(home.clone());
    }
    raw.push(std::env::temp_dir());
    raw.into_iter()
        .map(|p| p.canonicalize().unwrap_or(p))
        .collect()
}

pub fn validate_read_path(path: &Path, working_dir: &Path) -> Result<PathBuf, String> {
    let canonical = if path.exists() {
        path.canonicalize().map_err(sanitize_error)?
    } else {
        let parent = path.parent().ok_or("Chemin invalide")?;
        let filename = path.file_name().ok_or("Chemin sans nom de fichier")?;
        let canonical_parent = if parent.as_os_str().is_empty() {
            working_dir.canonicalize().map_err(sanitize_error)?
        } else {
            parent.canonicalize().map_err(sanitize_error)?
        };
        canonical_parent.join(filename)
    };

    let working_canonical = working_dir.canonicalize().unwrap_or_else(|_| working_dir.to_path_buf());
    if canonical.starts_with(&working_canonical) {
        return Ok(canonical);
    }

    let roots = allowed_read_roots();
    if roots.iter().any(|r| canonical.starts_with(r)) {
        Ok(canonical)
    } else {
        Err("Lecture interdite hors des zones autorisées".into())
    }
}

pub fn validate_write_path(path: &Path) -> Result<PathBuf, String> {
    let canonical = if path.exists() {
        path.canonicalize().map_err(sanitize_error)?
    } else {
        let parent = path.parent().ok_or("Chemin invalide")?;
        let filename = path.file_name().ok_or("Chemin sans nom de fichier")?;
        let canonical_parent = if parent.as_os_str().is_empty() {
            std::env::current_dir().map_err(sanitize_error)?
        } else {
            parent.canonicalize().map_err(sanitize_error)?
        };
        canonical_parent.join(filename)
    };

    let roots = allowed_write_roots();
    if roots.iter().any(|r| canonical.starts_with(r)) {
        Ok(canonical)
    } else {
        Err(
            "Écriture interdite hors des zones autorisées (data, .ollama, temp, Projects)"
                .into(),
        )
    }
}

pub fn sanitize_error<E: std::fmt::Display>(err: E) -> String {
    let msg = err.to_string();
    if msg.contains("No such file") || msg.contains("not found") {
        "Fichier introuvable".into()
    } else if msg.contains("Permission denied") {
        "Permission refusée".into()
    } else if msg.contains("Is a directory") {
        "Le chemin est un dossier".into()
    } else if msg.contains("Not a directory") {
        "Le chemin n'est pas un dossier".into()
    } else {
        "Erreur système".into()
    }
}

#[path = "security_tests.rs"]
#[cfg(test)]
mod tests;
