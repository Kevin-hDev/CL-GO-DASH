#[cfg(test)]
mod tests {
    use crate::services::llm::model_registry::{
        is_body_size_ok, is_trusted_host, parse_registry, MAX_BODY_BYTES, MAX_REGISTRY_ENTRIES,
    };

    fn fake_entry() -> String {
        r#"{"litellm_provider":"openai","mode":"chat"}"#.to_string()
    }

    fn build_json(count: usize) -> String {
        let entries: Vec<String> = (0..count)
            .map(|i| format!(r#""model-{i}": {}"#, fake_entry()))
            .collect();
        format!("{{{}}}", entries.join(","))
    }

    #[test]
    fn parses_valid_json() {
        let json = build_json(10);
        let map = parse_registry(&json);
        assert_eq!(map.len(), 10);
    }

    #[test]
    fn rejects_invalid_json() {
        let map = parse_registry("not json at all");
        assert!(map.is_empty());
    }

    #[test]
    fn skips_malformed_entries() {
        let json = r#"{"good": {"litellm_provider":"x","mode":"chat"}, "bad": "not an object"}"#;
        let map = parse_registry(json);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("good"));
    }

    #[test]
    fn enforces_max_entries() {
        let over_limit = MAX_REGISTRY_ENTRIES + 500;
        let json = build_json(over_limit);
        let map = parse_registry(&json);
        assert_eq!(map.len(), MAX_REGISTRY_ENTRIES);
    }

    #[test]
    fn exact_limit_accepted() {
        let json = build_json(MAX_REGISTRY_ENTRIES);
        let map = parse_registry(&json);
        assert_eq!(map.len(), MAX_REGISTRY_ENTRIES);
    }

    #[test]
    fn under_limit_accepted() {
        let json = build_json(100);
        let map = parse_registry(&json);
        assert_eq!(map.len(), 100);
    }

    #[test]
    fn empty_json_object() {
        let map = parse_registry("{}");
        assert!(map.is_empty());
    }

    #[test]
    fn trusted_host_github() {
        assert!(is_trusted_host("raw.githubusercontent.com"));
    }

    #[test]
    fn rejects_unknown_host() {
        assert!(!is_trusted_host("evil.com"));
        assert!(!is_trusted_host("raw.githubusercontent.com.evil.com"));
        assert!(!is_trusted_host(""));
    }

    #[test]
    fn body_size_within_limit() {
        assert!(is_body_size_ok(0));
        assert!(is_body_size_ok(1024));
        assert!(is_body_size_ok(MAX_BODY_BYTES));
    }

    #[test]
    fn body_size_over_limit() {
        assert!(!is_body_size_ok(MAX_BODY_BYTES + 1));
        assert!(!is_body_size_ok(usize::MAX));
    }
}
