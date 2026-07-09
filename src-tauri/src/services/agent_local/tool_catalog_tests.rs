use super::tool_catalog::*;

#[test]
fn defaults_match_product_choice() {
    assert_eq!(
        default_enabled_optional_tools(),
        vec![
            "load_skill",
            "ask_user_choice",
            "delegate_task",
            "list_subagents",
            "get_subagent",
            "cancel_subagent",
            "message_subagent",
            "planmode",
            "exitplanmode"
        ]
    );
}

#[test]
fn rejects_locked_and_unknown_tool_ids() {
    assert!(validate_optional_tool_id("bash").is_err());
    assert!(validate_optional_tool_id("missing_tool").is_err());
    assert!(validate_optional_tool_id("load_skill").is_ok());
}

#[test]
fn filtered_definitions_keep_locked_and_enabled_optional_tools() {
    let enabled = vec!["load_skill".to_string()];
    let defs = super::tool_definitions::get_tool_definitions();
    let names = tool_names(&filter_tool_definitions(defs, &enabled));

    assert!(has_tool(&names, "bash"));
    assert!(has_tool(&names, "search_mcp_tools"));
    assert!(has_tool(&names, "load_skill"));
    assert!(!has_tool(&names, "todo_write"));
    assert!(!has_tool(&names, "forecast"));
}

#[test]
fn delegate_task_enables_all_subagent_control_tools() {
    let enabled = normalize_enabled_optional_tools(&["delegate_task".to_string()]);
    for tool_id in SUBAGENT_TOOLS {
        assert!(enabled.iter().any(|id| id == tool_id));
    }
}
