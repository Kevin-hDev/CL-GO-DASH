use crate::services::agent_local::skill_catalog;
use crate::services::agent_local::types_tools::SkillInfo;

const MAX_SKILL_ID_BYTES: usize = 768;

pub async fn list_skills() -> Result<Vec<SkillInfo>, String> {
    tokio::task::spawn_blocking(|| {
        skill_catalog::entries().map(|entries| {
            entries
                .into_iter()
                .map(|entry| entry.info)
                .collect::<Vec<_>>()
        })
    })
    .await
    .map_err(|_| "Skills indisponibles".to_string())?
}

pub async fn load_skill(skill_id: &str) -> Result<String, String> {
    if skill_id.is_empty()
        || skill_id.len() > MAX_SKILL_ID_BYTES
        || skill_id.contains("..")
        || skill_id
            .chars()
            .any(|value| matches!(value, '/' | '\\' | '\0'))
    {
        return Err("Identifiant de skill invalide".into());
    }
    let requested = skill_id.to_string();
    tokio::task::spawn_blocking(move || {
        let entry = skill_catalog::entries()?
            .into_iter()
            .find(|entry| entry.info.id == requested)
            .ok_or_else(|| "Skill indisponible".to_string())?;
        let metadata =
            std::fs::metadata(&entry.manifest).map_err(|_| "Skill indisponible")?;
        if !metadata.is_file() || metadata.len() > 256 * 1024 {
            return Err("Skill indisponible".to_string());
        }
        let content =
            std::fs::read_to_string(&entry.manifest).map_err(|_| "Skill indisponible")?;
        let (_, _, body) =
            crate::services::agent_local::skill_parser::parse_skill_content(&content, &entry.info.name);
        Ok(format!(
            "Skill source: {}\nSkill directory: {}\n\n{}",
            entry.info.source_name,
            entry.bundle_root.display(),
            body
        ))
    })
    .await
    .map_err(|_| "Skill indisponible".to_string())?
}
