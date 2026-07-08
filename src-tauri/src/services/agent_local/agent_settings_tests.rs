use super::*;
use crate::services::agent_local::tool_catalog;

#[test]
fn old_settings_without_tools_get_product_defaults() {
    let settings: AgentSettings = serde_json::from_str(r#"{"permission_mode":"manual"}"#).unwrap();
    let settings = settings.normalized();

    assert_eq!(settings.permission_mode, "manual");
    assert_eq!(
        settings.enabled_optional_tools,
        tool_catalog::default_enabled_optional_tools()
    );
}

#[test]
fn permission_mode_change_preserves_enabled_tools() {
    let settings = AgentSettings {
        permission_mode: "auto".to_string(),
        enabled_optional_tools: vec!["load_skill".to_string()],
    };

    let updated = with_permission_mode(settings, "manual".to_string()).unwrap();

    assert_eq!(updated.permission_mode, "manual");
    assert_eq!(updated.enabled_optional_tools, vec!["load_skill"]);
}

#[test]
fn grouped_toggle_updates_all_real_tools() {
    let settings = AgentSettings {
        permission_mode: "auto".to_string(),
        enabled_optional_tools: vec!["planmode".to_string(), "exitplanmode".to_string()],
    };

    let updated = with_tool_group_enabled(settings, "plan_mode", false).unwrap();

    assert!(updated.enabled_optional_tools.is_empty());
}

#[test]
fn old_delegate_setting_enables_all_subagent_tools() {
    let settings = AgentSettings {
        permission_mode: "auto".to_string(),
        enabled_optional_tools: vec!["delegate_task".to_string()],
    }
    .normalized();

    for tool_id in tool_catalog::SUBAGENT_TOOLS {
        assert!(
            settings
                .enabled_optional_tools
                .iter()
                .any(|id| id == tool_id),
            "{tool_id} should be enabled with delegate_task"
        );
    }
}

#[test]
fn disabling_subagents_removes_all_control_tools() {
    let settings = AgentSettings {
        permission_mode: "auto".to_string(),
        enabled_optional_tools: tool_catalog::SUBAGENT_TOOLS
            .iter()
            .map(|id| (*id).to_string())
            .collect(),
    };

    let updated = with_tool_group_enabled(settings, "subagents", false).unwrap();

    for tool_id in tool_catalog::SUBAGENT_TOOLS {
        assert!(!updated
            .enabled_optional_tools
            .iter()
            .any(|id| id == tool_id));
    }
}

#[test]
fn grouped_toggle_rejects_locked_group() {
    let settings = AgentSettings::default();

    assert!(with_tool_group_enabled(settings, "web", false).is_err());
}
