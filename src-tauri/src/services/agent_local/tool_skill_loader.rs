use crate::services::agent_local::skill_parser::parse_skill_content;
use crate::services::agent_local::types_tools::SkillInfo;
use std::path::PathBuf;

const SKILL_FILENAMES: &[&str] = &["skill.md", "SKILL.md"];

fn skills_dir() -> PathBuf {
    crate::services::paths::data_dir().join("skills")
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
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let Some(file_path) = find_skill_file(&path) else {
            continue;
        };
        let content = tokio::fs::read_to_string(&file_path)
            .await
            .unwrap_or_default();
        let (name, description, _) = parse_skill_content(&content, &dir_name);
        skills.push(SkillInfo {
            name,
            description,
            path: path.to_string_lossy().to_string(),
            source: "user".to_string(),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub async fn load_skill(name: &str) -> Result<String, String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Nom de skill invalide".into());
    }
    let dir = skills_dir();
    let skill_dir = find_skill_dir_by_name(&dir, name)
        .await
        .ok_or_else(|| format!("Skill '{name}' non trouvé"))?;
    let skill_file = find_skill_file(&skill_dir)
        .ok_or_else(|| format!("Skill '{name}' non trouvé"))?;
    let content = tokio::fs::read_to_string(&skill_file)
        .await
        .map_err(|e| format!("Erreur lecture skill: {e}"))?;
    let (_, _, body) = parse_skill_content(&content, name);
    Ok(body)
}

async fn find_skill_dir_by_name(
    skills_root: &std::path::Path,
    name: &str,
) -> Option<PathBuf> {
    let mut read_dir = tokio::fs::read_dir(skills_root).await.ok()?;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if dir_name == name {
            return Some(path);
        }
        let Some(skill_file) = find_skill_file(&path) else {
            continue;
        };
        let content = tokio::fs::read_to_string(&skill_file).await.ok()?;
        let (parsed_name, _, _) = parse_skill_content(&content, &dir_name);
        if parsed_name == name {
            return Some(path);
        }
    }
    None
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
