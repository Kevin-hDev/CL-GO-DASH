use std::path::Path;

pub async fn load_agent_md(project_dir: Option<&Path>) -> Option<String> {
    load_agent_md_from(crate::services::paths::data_dir().as_path(), project_dir).await
}

pub async fn load_agent_md_from(
    data_dir: &Path,
    project_dir: Option<&Path>,
) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();

    let global_path = data_dir.join("AGENT.md");
    if let Ok(content) = tokio::fs::read_to_string(&global_path).await {
        let content = content.trim().to_string();
        if !content.is_empty() {
            sections.push(format!(
                "Contents of {} (global instructions):\n\n{}",
                global_path.display(),
                content,
            ));
        }
    }

    if let Some(proj) = project_dir {
        let proj_path = proj.join("AGENT.md");
        if let Ok(content) = tokio::fs::read_to_string(&proj_path).await {
            let content = content.trim().to_string();
            if !content.is_empty() {
                sections.push(format!(
                    "Contents of {} (project instructions):\n\n{}",
                    proj_path.display(),
                    content,
                ));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn load_global_only() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("AGENT.md");
        tokio::fs::write(&global, "# Global rules\nBe concise.").await.unwrap();

        let result = load_agent_md_from(tmp.path(), None).await;
        assert!(result.is_some());
        let content = result.unwrap();
        assert!(content.contains("Global rules"));
        assert!(content.contains("global instructions"));
    }

    #[tokio::test]
    async fn load_global_and_project() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("AGENT.md");
        tokio::fs::write(&global, "Global stuff").await.unwrap();

        let proj = TempDir::new().unwrap();
        let proj_md = proj.path().join("AGENT.md");
        tokio::fs::write(&proj_md, "Project stuff").await.unwrap();

        let result = load_agent_md_from(tmp.path(), Some(proj.path())).await;
        let content = result.unwrap();
        assert!(content.contains("Global stuff"));
        assert!(content.contains("Project stuff"));
        assert!(content.contains("project instructions"));
        let global_pos = content.find("Global stuff").unwrap();
        let project_pos = content.find("Project stuff").unwrap();
        assert!(project_pos > global_pos);
    }

    #[tokio::test]
    async fn no_files_returns_none() {
        let tmp = TempDir::new().unwrap();
        let result = load_agent_md_from(tmp.path(), Some(tmp.path())).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn project_only_no_global() {
        let tmp = TempDir::new().unwrap();
        let proj = TempDir::new().unwrap();
        let proj_md = proj.path().join("AGENT.md");
        tokio::fs::write(&proj_md, "Project only").await.unwrap();

        let result = load_agent_md_from(tmp.path(), Some(proj.path())).await;
        let content = result.unwrap();
        assert!(content.contains("Project only"));
        assert!(!content.contains("global instructions"));
    }
}
