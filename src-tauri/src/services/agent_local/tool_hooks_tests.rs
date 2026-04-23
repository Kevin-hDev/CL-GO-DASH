#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_hooks::{
        run_post_hooks, run_pre_hooks, PreHookDecision,
    };
    use crate::services::agent_local::types_tools::ToolResult;
    use serde_json::json;

    #[test]
    fn pre_hook_blocks_path_traversal() {
        let args = json!({ "path": "../etc/passwd" });
        match run_pre_hooks("write_file", &args) {
            PreHookDecision::Deny(_) => {}
            _ => panic!("Attendu Deny pour chemin avec '..'"),
        }
    }

    #[test]
    fn pre_hook_allows_normal_path() {
        let args = json!({ "path": "/home/user/project/file.txt" });
        match run_pre_hooks("write_file", &args) {
            PreHookDecision::Allow => {}
            _ => panic!("Attendu Allow pour chemin normal"),
        }
    }

    #[test]
    fn pre_hook_blocks_sensitive_bash() {
        let args = json!({ "command": "cat .env" });
        match run_pre_hooks("bash", &args) {
            PreHookDecision::Deny(_) => {}
            _ => panic!("Attendu Deny pour commande bash avec .env"),
        }
    }

    #[test]
    fn pre_hook_blocks_credentials_bash() {
        let args = json!({ "command": "cat credentials.json" });
        match run_pre_hooks("bash", &args) {
            PreHookDecision::Deny(_) => {}
            _ => panic!("Attendu Deny pour commande bash avec credentials"),
        }
    }

    #[test]
    fn pre_hook_blocks_id_rsa_bash() {
        let args = json!({ "command": "cat ~/.ssh/id_rsa" });
        match run_pre_hooks("bash", &args) {
            PreHookDecision::Deny(_) => {}
            _ => panic!("Attendu Deny pour commande bash avec id_rsa"),
        }
    }

    #[test]
    fn pre_hook_allows_normal_bash() {
        let args = json!({ "command": "ls -la" });
        match run_pre_hooks("bash", &args) {
            PreHookDecision::Allow => {}
            _ => panic!("Attendu Allow pour commande bash normale"),
        }
    }

    #[test]
    fn pre_hook_read_file_blocks_traversal() {
        let args = json!({ "path": "../../secret" });
        match run_pre_hooks("read_file", &args) {
            PreHookDecision::Deny(_) => {}
            _ => panic!("Attendu Deny pour read_file avec '..'"),
        }
    }

    #[test]
    fn pre_hook_edit_file_blocks_traversal() {
        let args = json!({ "path": "../config/../etc/shadow" });
        match run_pre_hooks("edit_file", &args) {
            PreHookDecision::Deny(_) => {}
            _ => panic!("Attendu Deny pour edit_file avec '..'"),
        }
    }

    #[test]
    fn post_hook_passes_through() {
        let result = ToolResult::ok("contenu du fichier");
        let args = json!({ "path": "/some/file.txt" });
        let after = run_post_hooks("read_file", &args, result.clone());
        assert_eq!(after.content, result.content);
        assert_eq!(after.is_error, result.is_error);
    }

    #[test]
    fn post_hook_passes_through_error() {
        let result = ToolResult::err("quelque chose a échoué");
        let args = json!({});
        let after = run_post_hooks("bash", &args, result.clone());
        assert!(after.is_error);
        assert_eq!(after.content, result.content);
    }
}
