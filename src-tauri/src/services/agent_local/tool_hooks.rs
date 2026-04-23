use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub enum PreHookDecision {
    Allow,
    AllowModified(Value),
    Deny(String),
}

const PROTECTED_FILES: &[&str] = &[
    "config.json",
    "secrets.enc",
    "agent-settings.json",
    "agent-tabs.json",
    "configured-providers.json",
];

fn is_protected_app_file(path_str: &str) -> bool {
    let path = std::path::Path::new(path_str);
    let data_dir = crate::services::paths::data_dir();
    let canonical = path.canonicalize().unwrap_or_else(|_| {
        path.parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.join(path.file_name().unwrap_or_default()))
            .unwrap_or_else(|| path.to_path_buf())
    });
    let data_canonical = data_dir.canonicalize().unwrap_or(data_dir);
    PROTECTED_FILES.iter().any(|f| canonical == data_canonical.join(f))
}

pub fn run_pre_hooks(tool_name: &str, args: &Value) -> PreHookDecision {
    if matches!(tool_name, "write_file" | "edit_file" | "read_file") {
        if let Some(path) = args["path"].as_str() {
            if path.contains("..") {
                return PreHookDecision::Deny("Chemin avec '..' interdit".into());
            }
        }
    }

    if matches!(tool_name, "write_file" | "edit_file") {
        if let Some(path) = args["path"].as_str() {
            if is_protected_app_file(path) {
                return PreHookDecision::Deny(
                    "Écriture interdite sur les fichiers de configuration de l'application".into(),
                );
            }
        }
    }

    if tool_name == "bash" {
        if let Some(cmd) = args["command"].as_str() {
            if cmd.contains(".env") || cmd.contains("credentials") || cmd.contains("id_rsa") {
                return PreHookDecision::Deny(
                    "Commande accédant à des fichiers sensibles bloquée".into(),
                );
            }
            let data_dir = crate::services::paths::data_dir().to_string_lossy().to_string();
            for f in PROTECTED_FILES {
                if cmd.contains(&format!("{data_dir}/{f}")) {
                    return PreHookDecision::Deny(
                        "Commande ciblant un fichier de configuration protégé bloquée".into(),
                    );
                }
            }
        }
    }

    PreHookDecision::Allow
}

/// Hooks exécutés APRÈS chaque tool call.
/// Peut modifier le résultat (ex: filtrer des données sensibles).
pub fn run_post_hooks(
    _tool_name: &str,
    _args: &Value,
    result: ToolResult,
) -> ToolResult {
    result
}
