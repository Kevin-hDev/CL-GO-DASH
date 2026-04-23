use crate::services::agent_local::{
    tool_bash, tool_files, tool_glob, tool_grep, tool_web_fetch, tool_web_search,
};
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub use crate::services::agent_local::tool_definitions::{
    get_tool_definitions, get_chat_tool_definitions,
};

// Seuils de taille max par outil (en caractères)
const MAX_CHARS_BASH: usize = 30_000;
const MAX_CHARS_GREP: usize = 10_000;
const MAX_CHARS_GLOB: usize = 5_000;
const MAX_CHARS_WEB_FETCH: usize = 50_000;
const MAX_CHARS_WEB_SEARCH: usize = 10_000;
const MAX_CHARS_LIST_DIR: usize = 10_000;
const PREVIEW_SIZE: usize = 2_000;

/// Retourne le seuil max (en chars) pour un outil donné.
/// `None` = pas de troncature (ex: read_file qui gère lui-même l'offset).
fn max_chars_for_tool(name: &str) -> Option<usize> {
    match name {
        "bash" => Some(MAX_CHARS_BASH),
        "grep" => Some(MAX_CHARS_GREP),
        "glob" => Some(MAX_CHARS_GLOB),
        "web_fetch" => Some(MAX_CHARS_WEB_FETCH),
        "web_search" => Some(MAX_CHARS_WEB_SEARCH),
        "list_dir" => Some(MAX_CHARS_LIST_DIR),
        _ => None,
    }
}

/// Tronque le résultat si dépassement du seuil. Ne touche jamais les erreurs.
fn truncate_result(mut result: ToolResult, tool_name: &str) -> ToolResult {
    if result.is_error {
        return result;
    }
    let Some(max) = max_chars_for_tool(tool_name) else {
        return result;
    };
    let total = result.content.chars().count();
    if total <= max {
        return result;
    }

    // Preview UTF-8-safe : on prend PREVIEW_SIZE caractères au plus
    let preview: String = result.content.chars().take(PREVIEW_SIZE).collect();
    let omitted = total - PREVIEW_SIZE;
    let total_kb = total / 1024;

    result.content = format!(
        "[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]\n{preview}\n[{omitted} chars omis]"
    );
    result.truncated = true;
    result
}

async fn dispatch_inner(tool_name: &str, args: &Value, working_dir: &Path) -> ToolResult {
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("");
            let timeout = args["timeout"].as_u64();
            match tool_bash::execute_shell(cmd, working_dir, timeout).await {
                Ok(out) => {
                    let content = format!("{}\n{}", out.stdout, out.stderr).trim().to_string();
                    if out.exit_code != 0 {
                        ToolResult::err(content)
                    } else {
                        ToolResult::ok(content)
                    }
                }
                Err(e) => ToolResult::err(e),
            }
        }
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            let limit = args["limit"].as_u64().unwrap_or(tool_files::DEFAULT_LIMIT as u64) as usize;
            tool_files::read_file(path, working_dir, offset, limit).await
        }
        "write_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            tool_files::write_file(path, content, working_dir).await
        }
        "edit_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let old = args["old_string"].as_str().unwrap_or("");
            let new = args["new_string"].as_str().unwrap_or("");
            tool_files::edit_file(path, old, new, working_dir).await
        }
        "list_dir" => {
            let path = args["path"].as_str().unwrap_or(".");
            tool_files::list_dir(path, working_dir).await
        }
        "grep" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str();
            let glob_filter = args["glob"].as_str();
            tool_grep::grep(pattern, path, glob_filter, working_dir).await
        }
        "glob" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str();
            tool_glob::glob_files(pattern, path, working_dir).await
        }
        "web_search" => {
            let query = args["query"].as_str().unwrap_or("");
            match tool_web_search::web_search(query).await {
                Ok(results) => {
                    let text = results
                        .iter()
                        .map(|r| format!("**{}**\n{}\n{}", r.title, r.url, r.snippet))
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    ToolResult::ok(text)
                }
                Err(e) => ToolResult::err(e),
            }
        }
        "web_fetch" => {
            let url = args["url"].as_str().unwrap_or("");
            match tool_web_fetch::fetch_url(url).await {
                Ok(content) => ToolResult::ok(content),
                Err(e) => ToolResult::err(e),
            }
        }
        "load_skill" => {
            let name = args["skill_name"].as_str().unwrap_or("");
            match tool_skill_loader::load_skill(name).await {
                Ok(content) => ToolResult::ok(
                    format!("Skill '{name}' loaded. Follow its instructions:\n\n{content}")
                ),
                Err(e) => ToolResult::err(e),
            }
        }
        _ => ToolResult::err(format!("Outil inconnu: {tool_name}")),
    }
}

/// Injecte un hint correctif dans le résultat d'erreur selon le pattern détecté.
pub(crate) fn enrich_error(mut result: ToolResult, tool_name: &str) -> ToolResult {
    if !result.is_error {
        return result;
    }
    let hint = match tool_name {
        "edit_file" if result.content.contains("non trouvée") => {
            "\n\n[HINT: Utiliser read_file pour vérifier le contenu exact, puis copier-coller la chaîne exacte dans old_string]"
        }
        "edit_file" if result.content.contains("fois") => {
            "\n\n[HINT: old_string apparaît plusieurs fois. Ajouter plus de contexte (lignes avant/après) pour rendre la correspondance unique]"
        }
        "bash" if result.content.contains("command not found") => {
            "\n\n[HINT: Commande introuvable. Vérifier l'orthographe ou installer le paquet nécessaire]"
        }
        "bash" if result.content.contains("Timeout") => {
            "\n\n[HINT: Timeout dépassé. Augmenter le paramètre timeout ou utiliser une approche plus efficace]"
        }
        "write_file" | "edit_file" if result.content.contains("non lu") => {
            "\n\n[HINT: Le write guard exige de lire le fichier avant de le modifier. Appeler read_file d'abord]"
        }
        _ => "",
    };
    if !hint.is_empty() {
        result.content.push_str(hint);
    }
    result
}

pub async fn dispatch(tool_name: &str, args: &Value, working_dir: &Path) -> ToolResult {
    let result = dispatch_inner(tool_name, args, working_dir).await;
    let result = truncate_result(result, tool_name);
    enrich_error(result, tool_name)
}
