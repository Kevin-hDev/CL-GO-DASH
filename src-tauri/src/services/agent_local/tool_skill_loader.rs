use crate::services::agent_local::types_tools::SkillInfo;
use std::path::PathBuf;

const SKILL_FILENAMES: &[&str] = &["skill.md", "SKILL.md"];

fn skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".claude")
        .join("skills")
}

pub async fn list_skills() -> Result<Vec<SkillInfo>, String> {
    let dir = skills_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut read_dir = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| format!("Erreur lecture skills: {e}"))?;

    let mut skills = Vec::new();
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let skill_file = find_skill_file(&path);
        let description = match &skill_file {
            Some(f) => extract_description(f).await,
            None => String::new(),
        };
        skills.push(SkillInfo {
            name,
            description,
            path: path.to_string_lossy().to_string(),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub async fn load_skill(name: &str) -> Result<String, String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Nom de skill invalide".into());
    }
    let dir = skills_dir().join(name);
    let skill_file = find_skill_file(&dir)
        .ok_or_else(|| format!("Skill '{name}' non trouvé"))?;
    tokio::fs::read_to_string(&skill_file)
        .await
        .map_err(|e| format!("Erreur lecture skill: {e}"))
}

fn find_skill_file(dir: &std::path::Path) -> Option<PathBuf> {
    for filename in SKILL_FILENAMES {
        let path = dir.join(filename);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

async fn extract_description(path: &std::path::Path) -> String {
    let content = tokio::fs::read_to_string(path).await.unwrap_or_default();
    content
        .lines()
        .find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("---"))
        .unwrap_or("")
        .chars()
        .take(100)
        .collect()
}
