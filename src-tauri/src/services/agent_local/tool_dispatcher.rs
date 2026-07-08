use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::{
    tool_bash, tool_files, tool_glob, tool_grep, tool_validate, tool_web_fetch, tool_web_search,
};
use serde_json::Value;
use std::path::Path;

pub use crate::services::agent_local::tool_definitions::get_tool_definitions;
pub use crate::services::agent_local::tool_definitions_chat::get_chat_tool_definitions;

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
                        if let Err(e) =
                            crate::services::agent_local::session_store::update_working_dir(
                                session_id, cwd,
                            )
                            .await
                        {
                            return ToolResult::err(format!(
                                "Commande exécutée, mais mise à jour du dossier courant impossible: {e}"
                            ));
                        }
                    }
                    let content = format!("{}\n{}", out.stdout, out.stderr).trim().to_string();
                    if out.exit_code != 0 {
                        ToolResult::err(content)
                    } else {
                        ToolResult::ok(content).with_affected_paths(out.affected_paths)
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
                Err(e) => ToolResult::err(e.to_string()),
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
            super::tool_dispatcher_delegate::dispatch_delegate(args, session_id).await
        }
        _ => {
            if let Some(result) =
                super::tool_subagent_control::dispatch(tool_name, args, session_id).await
            {
                return result;
            }
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
    if super::tool_catalog::is_optional_tool(tool_name)
        && !super::agent_settings::is_tool_enabled(tool_name).await
    {
        return ToolResult::err("Outil désactivé dans les paramètres.");
    }
    let args = match tool_validate::validate(tool_name, args) {
        Ok(cleaned) => cleaned,
        Err(msg) => return ToolResult::err(format!("[{tool_name}] {msg}")),
    };
    let result = dispatch_inner(tool_name, &args, working_dir, session_id).await;
    let result = super::tool_result_truncate::truncate_result(result, tool_name, session_id);
    enrich_error(result, tool_name)
}
