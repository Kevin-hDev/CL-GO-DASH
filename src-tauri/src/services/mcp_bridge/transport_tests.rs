#[cfg(test)]
mod tests {
    use super::super::transport::*;
    use serde_json::json;

    fn make_tool(name: &str, desc: Option<&str>, schema: Option<serde_json::Value>) -> McpToolDef {
        McpToolDef {
            name: name.to_string(),
            description: desc.map(|s| s.to_string()),
            input_schema: schema,
        }
    }

    #[test]
    fn name_valid_kept() {
        let mut t = make_tool("list_issues", None, None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.name, "list_issues");
    }

    #[test]
    fn name_special_chars_stripped() {
        let mut t = make_tool("list<issues>", None, None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.name, "listissues");
    }

    #[test]
    fn name_truncated_at_64() {
        let long_name = "a".repeat(100);
        let mut t = make_tool(&long_name, None, None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.name.len(), MAX_NAME_CHARS);
    }

    #[test]
    fn name_empty_after_strip() {
        let mut t = make_tool("!!!", None, None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.name, "");
    }

    #[test]
    fn desc_normal_kept() {
        let mut t = make_tool("tool", Some("Does stuff"), None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.description.as_deref(), Some("Does stuff"));
    }

    #[test]
    fn desc_control_chars_stripped() {
        let mut t = make_tool("tool", Some("foo\x00bar\tbaz"), None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.description.as_deref(), Some("foobarbaz"));
    }

    #[test]
    fn desc_newline_kept() {
        let mut t = make_tool("tool", Some("line1\nline2"), None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.description.as_deref(), Some("line1\nline2"));
    }

    #[test]
    fn desc_truncated_at_250() {
        let long_desc = "x".repeat(500);
        let mut t = make_tool("tool", Some(&long_desc), None);
        sanitize_tool_def(&mut t);
        assert_eq!(t.description.as_ref().unwrap().len(), MAX_DESC_CHARS);
    }

    #[test]
    fn schema_shallow_kept() {
        let schema = json!({"type": "object", "properties": {"a": {"type": "string"}}});
        let mut t = make_tool("tool", None, Some(schema.clone()));
        sanitize_tool_def(&mut t);
        assert!(t.input_schema.is_some());
    }

    #[test]
    fn schema_deep_dropped() {
        let schema = json!({"a": {"b": {"c": {"d": {"e": "leaf"}}}}});
        let mut t = make_tool("tool", None, Some(schema));
        sanitize_tool_def(&mut t);
        assert!(t.input_schema.is_none());
    }

    #[test]
    fn schema_many_props_dropped() {
        let props: serde_json::Map<String, serde_json::Value> = (0..25)
            .map(|i| (format!("prop{i}"), json!("string")))
            .collect();
        let schema = serde_json::Value::Object(props);
        let mut t = make_tool("tool", None, Some(schema));
        sanitize_tool_def(&mut t);
        assert!(t.input_schema.is_none());
    }

    #[test]
    fn schema_within_limits_kept() {
        let schema = json!({"type": "object", "properties": {"a": {"type": "string"}, "b": {"type": "number"}}});
        let mut t = make_tool("tool", None, Some(schema));
        sanitize_tool_def(&mut t);
        assert!(t.input_schema.is_some());
    }

    #[test]
    fn tools_over_max_truncated() {
        let tools: Vec<McpToolDef> = (0..130)
            .map(|i| make_tool(&format!("tool{i}"), None, None))
            .collect();
        let result = sanitize_tools(tools);
        assert_eq!(result.len(), MAX_TOOLS);
    }

    #[test]
    fn tools_empty_name_filtered() {
        let tools = vec![
            make_tool("valid_tool", None, None),
            make_tool("!!!", None, None),
        ];
        let result = sanitize_tools(tools);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "valid_tool");
    }

    #[test]
    fn tools_normal_pass_through() {
        let tools = vec![
            make_tool("tool_one", None, None),
            make_tool("tool-two", None, None),
            make_tool("tool3", None, None),
        ];
        let result = sanitize_tools(tools);
        assert_eq!(result.len(), 3);
    }

    // ── extract_tool_result ────────────────────────────────────────────────

    #[test]
    fn extract_error_message() {
        let resp = json!({"error": {"message": "not found"}});
        let err = extract_tool_result(&resp).unwrap_err();
        assert_eq!(err, "erreur MCP : not found");
    }

    #[test]
    fn extract_error_no_message() {
        let resp = json!({"error": {}});
        let err = extract_tool_result(&resp).unwrap_err();
        assert_eq!(err, "erreur MCP : erreur inconnue");
    }

    #[test]
    fn extract_text_content() {
        let resp = json!({"result": {"content": [{"text": "hello"}]}});
        let ok = extract_tool_result(&resp).unwrap();
        assert_eq!(ok, "hello");
    }

    #[test]
    fn extract_multi_text() {
        let resp = json!({"result": {"content": [{"text": "a"}, {"text": "b"}]}});
        let ok = extract_tool_result(&resp).unwrap();
        assert_eq!(ok, "a\nb");
    }

    #[test]
    fn extract_no_content() {
        let resp = json!({"result": {"data": 42}});
        let ok = extract_tool_result(&resp).unwrap();
        assert!(ok.contains("42"), "doit contenir la valeur JSON : {ok}");
    }

    #[test]
    fn extract_empty_result() {
        let resp = json!({});
        let err = extract_tool_result(&resp).unwrap_err();
        assert!(
            err.contains("réponse vide"),
            "message inattendu : {err}"
        );
    }
}
