use std::collections::HashSet;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 10;
const MAX_TOTAL_BYTES: usize = 50 * 1024; // 50 KB

pub async fn load_agent_md(cwd: Option<&Path>) -> Option<String> {
    load_agent_md_from(crate::services::paths::data_dir().as_path(), cwd).await
}

pub async fn load_agent_md_from(data_dir: &Path, cwd: Option<&Path>) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();
    let mut total_bytes: usize = 0;

    // 1. Global AGENT.md
    let global_path = data_dir.join("AGENT.md");
    if let Ok(content) = tokio::fs::read_to_string(&global_path).await {
        let content = content.trim().to_string();
        if !content.is_empty() {
            let entry = format!(
                "Contents of {} (global instructions):\n\n{}",
                global_path.display(),
                content
            );
            total_bytes += entry.len();
            sections.push(entry);
        }
    }

    // 2. Remontée hiérarchique depuis cwd
    if let Some(cwd_raw) = cwd {
        let ancestors = collect_ancestors(cwd_raw).await;

        // Du plus éloigné (racine) au plus proche (cwd) — les proches ont priorité
        for dir in ancestors.iter().rev() {
            if total_bytes >= MAX_TOTAL_BYTES {
                break;
            }
            collect_dir_entries(dir, &mut sections, &mut total_bytes).await;
        }
    }

    if sections.is_empty() {
        return None;
    }

    let header = "The following AGENT.md instructions define how you should behave. \
                  You MUST follow them exactly as written. These instructions OVERRIDE \
                  any default behavior.";

    Some(format!("{}\n\n{}", header, sections.join("\n\n")))
}

/// Remonte de `cwd` vers `/`, max MAX_DEPTH niveaux, dédupe via HashSet.
/// Retourne les dirs dans l'ordre du plus proche (index 0 = cwd) au plus éloigné.
async fn collect_ancestors(cwd_raw: &Path) -> Vec<PathBuf> {
    let start = match tokio::fs::canonicalize(cwd_raw).await {
        Ok(p) => p,
        Err(_) => cwd_raw.to_path_buf(),
    };

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut ancestors: Vec<PathBuf> = Vec::new();
    let mut current: PathBuf = start;

    for _ in 0..=MAX_DEPTH {
        if !visited.insert(current.clone()) {
            break; // Boucle symlink détectée
        }
        ancestors.push(current.clone());
        match current.parent() {
            Some(p) if p != current => current = p.to_path_buf(),
            _ => break, // Racine atteinte
        }
    }

    ancestors
}

/// Lit les fichiers AGENT.md d'un répertoire dans l'ordre :
/// 1. `AGENT.md` direct
/// 2. `.cl-go/AGENT.md`
/// 3. `.cl-go/rules/*.md` (tri alphabétique)
async fn collect_dir_entries(
    dir: &Path,
    sections: &mut Vec<String>,
    total_bytes: &mut usize,
) {
    // AGENT.md direct
    let agent_md = dir.join("AGENT.md");
    push_file(&agent_md, "project instructions", sections, total_bytes).await;

    // .cl-go/AGENT.md
    let cl_go_agent = dir.join(".cl-go").join("AGENT.md");
    push_file(&cl_go_agent, "project instructions", sections, total_bytes).await;

    // .cl-go/rules/*.md
    let rules_dir = dir.join(".cl-go").join("rules");
    if let Ok(mut entries) = tokio::fs::read_dir(&rules_dir).await {
        let mut rule_files: Vec<PathBuf> = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                rule_files.push(path);
            }
        }
        rule_files.sort();
        for rule_file in rule_files {
            push_file(&rule_file, "project rules", sections, total_bytes).await;
        }
    }
}

/// Lit un fichier et l'ajoute à sections si non vide et si la limite n'est pas atteinte.
async fn push_file(
    path: &Path,
    label: &str,
    sections: &mut Vec<String>,
    total_bytes: &mut usize,
) {
    if *total_bytes >= MAX_TOTAL_BYTES {
        return;
    }
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        let content = content.trim().to_string();
        if content.is_empty() {
            return;
        }
        let entry = format!("Contents of {} ({}):\n\n{}", path.display(), label, content);
        *total_bytes += entry.len();
        sections.push(entry);
    }
}

#[cfg(test)]
#[path = "agent_md_tests.rs"]
mod tests;
