use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ToolResult;
use globset::Glob;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

const MAX_RESULTS: usize = 100;

pub async fn glob_files(
    pattern: &str,
    path: Option<&str>,
    working_dir: &Path,
) -> ToolResult {
    let pattern = pattern.to_string();
    let root = resolve_root(path, working_dir);

    if let Err(e) = security::validate_read_path(&root, working_dir) {
        return ToolResult::err(e);
    }

    let result = tokio::task::spawn_blocking(move || glob_blocking(&pattern, &root)).await;
    match result {
        Ok(r) => r,
        Err(e) => ToolResult::err(format!("Erreur interne: {e}")),
    }
}

fn resolve_root(path: Option<&str>, working_dir: &Path) -> PathBuf {
    match path {
        Some(p) => {
            let pb = Path::new(p);
            if pb.is_absolute() { pb.to_path_buf() } else { working_dir.join(pb) }
        }
        None => working_dir.to_path_buf(),
    }
}

fn glob_blocking(pattern: &str, root: &Path) -> ToolResult {
    let matcher = match Glob::new(pattern) {
        Ok(g) => g.compile_matcher(),
        Err(e) => return ToolResult::err(format!("Pattern glob invalide : {e}")),
    };

    let walk = WalkBuilder::new(root).hidden(false).git_ignore(true).build();

    let mut results: Vec<String> = Vec::new();
    let mut truncated = false;

    for dent in walk {
        if results.len() >= MAX_RESULTS {
            truncated = true;
            break;
        }
        let entry = match dent {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        if matcher.is_match(rel) {
            results.push(rel.display().to_string());
        }
    }

    let mut output = results.join("\n");
    if truncated {
        output.push_str(&format!("\n... [tronqué à {MAX_RESULTS} résultats]"));
    }
    if output.is_empty() {
        output = "(aucun résultat)".into();
    }
    ToolResult::ok(output)
}
