#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_validate::validate;
    use serde_json::json;

    #[test]
    fn valid_bash() {
        let args = json!({"command": "ls", "timeout": 30});
        assert!(validate("bash", &args).is_ok());
    }

    #[test]
    fn bash_missing_command() {
        let args = json!({"timeout": 30});
        let err = validate("bash", &args).unwrap_err();
        assert!(err.contains("command"));
    }

    #[test]
    fn bash_wrong_type() {
        let args = json!({"command": 42});
        let err = validate("bash", &args).unwrap_err();
        assert!(err.contains("string"));
    }

    #[test]
    fn strips_unknown_args() {
        let args = json!({"command": "ls", "inject": "evil"});
        let cleaned = validate("bash", &args).unwrap();
        assert!(cleaned.get("inject").is_none());
        assert!(cleaned.get("command").is_some());
    }

    #[test]
    fn optional_args_absent() {
        let args = json!({"command": "ls"});
        assert!(validate("bash", &args).is_ok());
    }

    #[test]
    fn read_file_valid() {
        let args = json!({"path": "foo.rs", "offset": 0, "limit": 100});
        assert!(validate("read_file", &args).is_ok());
    }

    #[test]
    fn edit_file_missing_old_string() {
        let args = json!({"path": "f.rs", "new_string": "x"});
        let err = validate("edit_file", &args).unwrap_err();
        assert!(err.contains("old_string"));
    }

    #[test]
    fn search_mcp_valid() {
        let args = json!({"mode": "call", "tool_id": "svc.tool", "arguments": {"key": "val"}});
        assert!(validate("search_mcp_tools", &args).is_ok());
    }

    #[test]
    fn todo_write_requires_todos_array() {
        assert!(validate("todo_write", &json!({"todos": []})).is_ok());
        assert!(validate("todo_write", &json!({"todos": "nope"})).is_err());
    }

    #[test]
    fn hidden_todo_tools_validate_args() {
        assert!(validate("todo_history", &json!({})).is_ok());
        assert!(validate("agent_diagnostics", &json!({})).is_ok());
        assert!(validate("todo_pause", &json!({"reason": "debug"})).is_ok());
        assert!(validate("todo_resume", &json!({"id": "abc"})).is_ok());
        assert!(validate("todo_resume", &json!({})).is_err());
    }

    #[test]
    fn unknown_tool_passes_through() {
        let args = json!({"anything": true});
        assert!(validate("unknown_future_tool", &args).is_ok());
    }

    #[test]
    fn non_object_args_rejected() {
        let args = json!("not an object");
        assert!(validate("bash", &args).is_err());
    }

    #[test]
    fn write_spreadsheet_valid() {
        let args = json!({"path": "out.xlsx", "operations": [{"type": "set_cell"}]});
        assert!(validate("write_spreadsheet", &args).is_ok());
    }

    #[test]
    fn forecast_model_arg_is_ignored() {
        let args = json!({
            "target_column": "sales",
            "date_column": "date",
            "horizon": 7,
            "frequency": "D",
            "model": "chronos-bolt-small"
        });
        let cleaned = validate("forecast", &args).unwrap();

        assert!(cleaned.get("model").is_none());
    }

    #[test]
    fn null_required_arg_rejected() {
        let args = json!({"command": null});
        assert!(validate("bash", &args).is_err());
    }
}
