use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::{
    tool_bash, tool_files, tool_glob, tool_grep, tool_validate, tool_web_fetch, tool_web_search,
};
use crate::services::paths::data_dir;
use serde_json::Value;
use std::path::Path;

pub use crate::services::agent_local::tool_definitions::get_tool_definitions;
pub use crate::services::agent_local::tool_definitions_chat::get_chat_tool_definitions;

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
/// Quand tronqué, sauvegarde le résultat complet sur disque et inclut le chemin dans le message.
fn truncate_result(mut result: ToolResult, tool_name: &str, session_id: &str) -> ToolResult {
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

    // Sauvegarder le résultat complet sur disque
    let persist_path = persist_result(&result.content, session_id);

    // Preview UTF-8-safe : on prend PREVIEW_SIZE caractères au plus
    let preview: String = result.content.chars().take(PREVIEW_SIZE).collect();
    let omitted = total - PREVIEW_SIZE;
    let total_kb = total / 1024;

    let file_hint = match persist_path {
        Some(p) => format!("\n[Résultat complet disponible : {}]", p),
        None => String::new(),
    };

    result.content = format!(
        "[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]{file_hint}\n{preview}\n[{omitted} chars omis]"
    );
    result.truncated = true;
    result
}

/// Persiste le contenu complet dans data_dir()/tool-results/{session_id}/{uuid}.txt.
/// Retourne le chemin du fichier si la sauvegarde a réussi.
fn persist_result(content: &str, session_id: &str) -> Option<String> {
    let dir = data_dir().join("tool-results").join(session_id);
    std::fs::create_dir_all(&dir).ok()?;
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());
    let path = dir.join(&file_name);
    std::fs::write(&path, content).ok()?;
    Some(path.to_string_lossy().into_owned())
}

async fn dispatch_inner(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
) -> ToolResult {
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("");
            let timeout = args["timeout"].as_u64();
            match tool_bash::execute_shell(cmd, working_dir, timeout).await {
                Ok(out) => {
                    if let Some(ref cwd) = out.new_cwd {
                        let sid = session_id.to_string();
                        let dir = cwd.clone();
                        tokio::spawn(async move {
                            if let Err(e) =
                                crate::services::agent_local::session_store::update_working_dir(
                                    &sid, &dir,
                                )
                                .await
                            {
                                eprintln!("[cwd-track] échec update_working_dir: {e}");
                            }
                        });
                    }
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
            let limit = args["limit"]
                .as_u64()
                .unwrap_or(tool_files::DEFAULT_LIMIT as u64) as usize;
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
        "todo_write" => super::tool_todo::execute(args, session_id).await,
        "todo_history" => super::tool_todo::execute_history(args, session_id).await,
        "todo_pause" => super::tool_todo::execute_pause(args, session_id).await,
        "todo_resume" => super::tool_todo::execute_resume(args, session_id).await,
        "todo_delete" => super::tool_todo::execute_delete(args, session_id).await,
        "ask_user_choice" => ToolResult::err("Contexte interactif indisponible."),
        "planmode" | "exitplanmode" => ToolResult::err("Contexte plan indisponible."),
        "agent_diagnostics" => {
            let limit = args
                .get("limit")
                .and_then(|value| value.as_u64())
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or(super::stream_diagnostics_tools::DEFAULT_TOOL_LIMIT);
            match super::stream_diagnostics::diagnostics_text(session_id, limit).await {
                Ok(text) => ToolResult::ok(text),
                Err(_) => ToolResult::err("Diagnostics indisponibles."),
            }
        }
        "load_skill" => {
            let name = args["skill_name"].as_str().unwrap_or("");
            match tool_skill_loader::load_skill(name).await {
                Ok(content) => ToolResult::ok(format!(
                    "Skill '{name}' loaded. Follow its instructions:\n\n{content}"
                )),
                Err(e) => ToolResult::err(e),
            }
        }
        "create_branch" => {
            let branch_name = args["branch_name"].as_str().unwrap_or("");
            if branch_name.is_empty() {
                return ToolResult::err("Paramètre branch_name requis");
            }
            match crate::services::git::branch::create_branch(working_dir, branch_name) {
                Ok(()) => ToolResult::ok(format!(
                    "Branche '{}' créée et activée dans {}",
                    branch_name,
                    working_dir.display()
                )),
                Err(e) => ToolResult::err(e),
            }
        }
        "checkout_branch" => {
            let branch_name = args["branch_name"].as_str().unwrap_or("");
            if branch_name.is_empty() {
                return ToolResult::err("Paramètre branch_name requis");
            }
            match crate::services::git::branch::checkout_branch(working_dir, branch_name) {
                Ok(()) => ToolResult::ok(format!("Basculé sur la branche '{}'", branch_name)),
                Err(e) => ToolResult::err(e),
            }
        }
        "delegate_task" => {
            let Some(app) = super::app_handle_global::get() else {
                return ToolResult::err("AppHandle non initialisé");
            };
            let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::new(
                app.clone(),
                session_id.to_string(),
            );
            match super::tool_delegate::prepare_delegate(
                args.clone(),
                app.clone(),
                session_id.to_string(),
                emitter,
            )
            .await
            {
                Err(tr) => tr,
                Ok(spawned) => {
                    let msg = spawned.result_message.clone();
                    let child_id = spawned.child_id.clone();
                    if let Err(e) = super::subagent_spawn_channel::send(
                        super::subagent_spawn_channel::SpawnRequest {
                            app: spawned.app,
                            child_session_id: spawned.child_id,
                            model: spawned.model,
                            provider: spawned.provider,
                            prompt: spawned.prompt,
                            subagent_type: spawned.subagent_type,
                            parent_emitter: spawned.parent_emitter,
                            cancel: spawned.cancel,
                            project_id: spawned.project_id,
                        },
                    ) {
                        super::subagent_registry::unregister(&child_id).await;
                        let _ = super::session_subagents::mark_status(&child_id, "failed").await;
                        return ToolResult::err(e);
                    }
                    ToolResult::ok(msg)
                }
            }
        }
        _ => {
            if let Some(result) = super::tool_dispatcher_forecast::dispatch_forecast(
                tool_name,
                args,
                working_dir,
                session_id,
            )
            .await
            {
                return result;
            }
            if let Some(result) = super::tool_dispatcher_mcp::dispatch_mcp(tool_name, args).await {
                return result;
            }
            match super::tool_dispatcher_office::dispatch_office(
                tool_name,
                args,
                working_dir,
                session_id,
            )
            .await
            {
                Some(result) => result,
                None => ToolResult::err(format!("Outil inconnu: {tool_name}")),
            }
        }
    }
}

/// Injecte un hint correctif dans le résultat d'erreur selon le pattern détecté.
pub(crate) fn enrich_error(mut result: ToolResult, tool_name: &str) -> ToolResult {
    if !result.is_error {
        return result;
    }
    let hint = match tool_name {
        "edit_file" if result.content.contains("non trouvée") => "",
        "edit_file" if result.content.contains("fois") => {
            "\n\n[HINT: old_string apparaît plusieurs fois. Ajouter plus de contexte (lignes avant/après) pour rendre la correspondance unique]"
        }
        "bash" if result.content.contains("command not found") => {
            "\n\n[HINT: Commande introuvable. Vérifier l'orthographe ou installer le paquet nécessaire]"
        }
        "bash" if result.content.contains("Timeout") => {
            "\n\n[HINT: Timeout dépassé. Augmenter le paramètre timeout ou utiliser une approche plus efficace]"
        }
        _ => "",
    };
    if !hint.is_empty() {
        result.content.push_str(hint);
    }
    result
}

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
) -> ToolResult {
    let args = match tool_validate::validate(tool_name, args) {
        Ok(cleaned) => cleaned,
        Err(msg) => return ToolResult::err(format!("[{tool_name}] {msg}")),
    };
    let result = dispatch_inner(tool_name, &args, working_dir, session_id).await;
    let result = truncate_result(result, tool_name, session_id);
    enrich_error(result, tool_name)
}
