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
];

static S7_EVAL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"eval\s+"?\$"#).unwrap());

pub fn allowed_write_roots() -> Vec<PathBuf> {
    let mut raw = Vec::with_capacity(4);
    if let Some(home) = dirs::home_dir() {
        raw.push(home.join(".local/share/cl-go-dash"));
        raw.push(home.join(".ollama"));
        raw.push(home.join("Projects"));
    }
    raw.push(PathBuf::from("/tmp"));
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
            "Écriture interdite hors des zones autorisées (~/.local/share/cl-go-dash, ~/.ollama, /tmp, ~/Projects)"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_rm_rf_root() {
        assert!(check_destructive_command("rm -rf /").is_err());
        assert!(check_destructive_command("sudo rm -rf / --no-preserve-root").is_err());
    }

    #[test]
    fn blocks_rm_rf_wildcard() {
        assert!(check_destructive_command("rm -rf *").is_err());
    }

    #[test]
    fn blocks_sudo_rm() {
        assert!(check_destructive_command("sudo rm file.txt").is_err());
    }

    #[test]
    fn blocks_chmod_777() {
        assert!(check_destructive_command("chmod 777 file").is_err());
        assert!(check_destructive_command("chmod 777 /etc").is_err());
    }

    #[test]
    fn blocks_disk_operations() {
        assert!(check_destructive_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(check_destructive_command("mkfs.ext4 /dev/sda1").is_err());
        assert!(check_destructive_command("echo > /dev/sda").is_err());
        assert!(check_destructive_command("fdisk /dev/sda").is_err());
    }

    #[test]
    fn blocks_system_control() {
        assert!(check_destructive_command("shutdown now").is_err());
        assert!(check_destructive_command("reboot").is_err());
        assert!(check_destructive_command("init 0").is_err());
        assert!(check_destructive_command("init 6").is_err());
    }

    #[test]
    fn blocks_fork_bomb() {
        assert!(check_destructive_command(":(){:|:&};:").is_err());
    }

    #[test]
    fn blocks_eval_expansion() {
        assert!(check_destructive_command("eval $cmd").is_err());
        assert!(check_destructive_command(r#"eval "$user_input""#).is_err());
        assert!(check_destructive_command("eval  $var").is_err());
    }

    #[test]
    fn allows_safe_commands() {
        assert!(check_destructive_command("ls -la").is_ok());
        assert!(check_destructive_command("echo hello").is_ok());
        assert!(check_destructive_command("cat file.txt").is_ok());
        assert!(check_destructive_command("grep pattern *.rs").is_ok());
        assert!(check_destructive_command("eval 'echo static'").is_ok());
    }

    #[test]
    fn write_path_allows_tmp() {
        let p = Path::new("/tmp/test-cl-go-security.txt");
        let _ = std::fs::remove_file(p);
        assert!(validate_write_path(p).is_ok());
    }

    #[test]
    fn write_path_blocks_etc() {
        let p = Path::new("/etc/evil.conf");
        assert!(validate_write_path(p).is_err());
    }

    #[test]
    fn write_path_blocks_traversal() {
        let p = Path::new("/tmp/../etc/passwd");
        assert!(validate_write_path(p).is_err());
    }

    #[test]
    fn sanitize_masks_paths() {
        let e = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No such file or directory (os error 2): /Users/kevinh/secret",
        );
        assert_eq!(sanitize_error(e), "Fichier introuvable");
    }
}
