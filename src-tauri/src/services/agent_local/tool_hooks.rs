use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub enum PreHookDecision {
    Allow,
    AllowModified(Value),
    Deny(String),
}

/// Hooks exécutés AVANT chaque tool call.
/// Retourne Allow, AllowModified (args modifiés), ou Deny (bloque).
pub fn run_pre_hooks(tool_name: &str, args: &Value) -> PreHookDecision {
    // Hook : bloquer les chemins avec ".." dans les tools fichier
    if matches!(tool_name, "write_file" | "edit_file" | "read_file") {
        if let Some(path) = args["path"].as_str() {
            if path.contains("..") {
                return PreHookDecision::Deny(
                    "Chemin avec '..' interdit".to_string(),
                );
            }
        }
    }

    // Hook : bloquer bash avec des accès à des fichiers sensibles
    if tool_name == "bash" {
        if let Some(cmd) = args["command"].as_str() {
            if cmd.contains(".env")
                || cmd.contains("credentials")
                || cmd.contains("id_rsa")
            {
                return PreHookDecision::Deny(
                    "Commande accédant à des fichiers sensibles bloquée"
                        .to_string(),
                );
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
