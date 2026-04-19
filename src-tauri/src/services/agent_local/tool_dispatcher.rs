use crate::services::agent_local::{
    tool_bash, tool_files, tool_glob, tool_grep, tool_web_fetch, tool_web_search,
};
use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub async fn dispatch(tool_name: &str, args: &Value, working_dir: &Path) -> ToolResult {
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("");
            let timeout = args["timeout"].as_u64();
            match tool_bash::execute_shell(cmd, working_dir, timeout).await {
                Ok(out) => ToolResult {
                    content: format!("{}\n{}", out.stdout, out.stderr).trim().to_string(),
                    is_error: out.exit_code != 0,
                },
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("");
            tool_files::read_file(path, working_dir).await
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
                    ToolResult { content: text, is_error: false }
                }
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        "web_fetch" => {
            let url = args["url"].as_str().unwrap_or("");
            match tool_web_fetch::fetch_url(url).await {
                Ok(content) => ToolResult { content, is_error: false },
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        "load_skill" => {
            let name = args["skill_name"].as_str().unwrap_or("");
            match tool_skill_loader::load_skill(name).await {
                Ok(content) => ToolResult {
                    content: format!("Skill '{name}' loaded. Follow its instructions:\n\n{content}"),
                    is_error: false,
                },
                Err(e) => ToolResult { content: e, is_error: true },
            }
        }
        _ => ToolResult {
            content: format!("Outil inconnu: {tool_name}"),
            is_error: true,
        },
    }
}

pub async fn dispatch_multiple(
    tool_calls: &[(String, Value)],
    working_dir: &Path,
) -> Vec<ToolResult> {
    let futures: Vec<_> = tool_calls
        .iter()
        .map(|(name, args)| dispatch(name, args, working_dir))
        .collect();
    futures_util::future::join_all(futures).await
}

pub fn get_tool_definitions() -> Vec<Value> {
    vec![
        tool_def("bash", "Exécuter une commande shell (bash)", serde_json::json!({
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "Commande à exécuter"},
                "timeout": {"type": "integer", "description": "Timeout en secondes"}
            },
            "required": ["command"]
        })),
        tool_def("read_file", "Lire le contenu d'un fichier", serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Chemin du fichier"}
            },
            "required": ["path"]
        })),
        tool_def("write_file", "Écrire dans un fichier", serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Chemin du fichier"},
                "content": {"type": "string", "description": "Contenu à écrire"}
            },
            "required": ["path", "content"]
        })),
        tool_def("edit_file", "Modifier un fichier (remplacement exact)", serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Chemin du fichier"},
                "old_string": {"type": "string", "description": "Texte à remplacer (unique)"},
                "new_string": {"type": "string", "description": "Nouveau texte"}
            },
            "required": ["path", "old_string", "new_string"]
        })),
        tool_def("list_dir", "Lister le contenu d'un répertoire", serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Chemin du répertoire"}
            },
            "required": ["path"]
        })),
        tool_def("grep", "Rechercher un motif regex dans les fichiers (max 250 résultats, respecte .gitignore)", serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Motif regex à rechercher"},
                "path": {"type": "string", "description": "Dossier de recherche (défaut: working dir)"},
                "glob": {"type": "string", "description": "Filtre glob sur les fichiers (ex: '*.rs')"}
            },
            "required": ["pattern"]
        })),
        tool_def("glob", "Lister les fichiers correspondant à un motif glob (max 100 résultats, respecte .gitignore)", serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "Motif glob (ex: '**/*.ts', 'src/**/*.rs')"},
                "path": {"type": "string", "description": "Racine de recherche (défaut: working dir)"}
            },
            "required": ["pattern"]
        })),
        tool_def("web_search", "Rechercher sur le web", serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Requête de recherche"}
            },
            "required": ["query"]
        })),
        tool_def("web_fetch", "Récupérer le contenu d'une URL", serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "URL à récupérer"}
            },
            "required": ["url"]
        })),
        tool_def("load_skill", "Charger un skill par son nom. Utilise cet outil quand l'utilisateur mentionne un skill ou que la tâche correspond à un skill disponible.", serde_json::json!({
            "type": "object",
            "properties": {
                "skill_name": {"type": "string", "description": "Nom exact du skill à charger (tel qu'affiché dans la liste des skills disponibles)"}
            },
            "required": ["skill_name"]
        })),
    ]
}

fn tool_def(name: &str, description: &str, parameters: Value) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}
