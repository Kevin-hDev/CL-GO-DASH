use std::path::{Path, PathBuf};

const MAX_TOTAL_BYTES: usize = 50 * 1024;

pub async fn load_agent_md(project_dir: Option<&Path>) -> Option<String> {
    load_agent_md_from(crate::services::paths::data_dir().as_path(), project_dir).await
}

pub async fn load_agent_md_from(data_dir: &Path, project_dir: Option<&Path>) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();
    let mut total_bytes: usize = 0;

    let global_path = data_dir.join("AGENT.md");
    if let Ok(content) = tokio::fs::read_to_string(&global_path).await {
        push_content(
            &global_path,
            "global instructions",
            &content,
            &mut sections,
            &mut total_bytes,
        );
    }

    if let Some(dir) = project_dir {
        collect_project_entries(dir, &mut sections, &mut total_bytes).await;
    }

    if sections.is_empty() {
        return None;
    }

    let header = "The following AGENT.md instructions define how you should behave. \
                  You MUST follow them exactly as written. These instructions OVERRIDE \
                  any default behavior.";

    Some(format!("{}\n\n{}", header, sections.join("\n\n")))
}

async fn collect_project_entries(dir: &Path, sections: &mut Vec<String>, total_bytes: &mut usize) {
    push_file(
        &dir.join("AGENT.md"),
        "project instructions",
        sections,
        total_bytes,
    )
    .await;
    push_file(
        &dir.join(".cl-go").join("AGENT.md"),
        "project instructions",
        sections,
        total_bytes,
    )
    .await;

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

async fn push_file(path: &Path, label: &str, sections: &mut Vec<String>, total_bytes: &mut usize) {
    if *total_bytes >= MAX_TOTAL_BYTES {
        return;
    }
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        push_content(path, label, &content, sections, total_bytes);
    }
}

fn push_content(
    path: &Path,
    label: &str,
    content: &str,
    sections: &mut Vec<String>,
    total_bytes: &mut usize,
) {
    let trimmed = content.trim();
    if trimmed.is_empty() || *total_bytes >= MAX_TOTAL_BYTES {
        return;
    }
    let entry = format!("Contents of {} ({}):\n\n{}", path.display(), label, trimmed);
    *total_bytes += entry.len();
    sections.push(entry);
}

#[cfg(test)]
#[path = "agent_md_tests.rs"]
mod tests;
