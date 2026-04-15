use crate::services::agent_local::agent_settings::{self, AgentSettings};
use crate::services::agent_local::permission_gate;

#[tauri::command]
pub async fn get_agent_settings() -> Result<AgentSettings, String> {
    Ok(agent_settings::load().await)
}

#[tauri::command]
pub async fn set_permission_mode(mode: String) -> Result<(), String> {
    let settings = AgentSettings {
        permission_mode: mode,
    };
    agent_settings::save(&settings).await
}

#[tauri::command]
pub async fn respond_to_permission(id: String, decision: String) -> Result<(), String> {
    let parsed = match decision.as_str() {
        "allow" => permission_gate::PermissionDecision::Allow,
        "allow_session" => permission_gate::PermissionDecision::AllowSession,
        "deny" => permission_gate::PermissionDecision::Deny,
        other => return Err(format!("Décision inconnue: {other}")),
    };
    permission_gate::respond(&id, parsed).await;
    Ok(())
}
