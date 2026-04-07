use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::tool_web_search;
use crate::services::agent_local::types_tools::SkillInfo;

#[tauri::command]
pub async fn list_skills() -> Result<Vec<SkillInfo>, String> {
    tool_skill_loader::list_skills().await
}

#[tauri::command]
pub async fn load_skill(name: String) -> Result<String, String> {
    tool_skill_loader::load_skill(&name).await
}

#[tauri::command]
pub async fn set_brave_api_key(key: String) -> Result<(), String> {
    tool_web_search::set_brave_key(&key)
}
