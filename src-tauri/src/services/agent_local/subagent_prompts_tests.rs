#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_prompts::explorer_system;

    #[test]
    fn test_explorer_system_contains_role() {
        let prompt = explorer_system();
        assert!(
            prompt.contains("<role>"),
            "Le prompt explorer doit contenir la balise <role>"
        );
    }

    #[test]
    fn test_explorer_system_contains_date() {
        let prompt = explorer_system();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(
            prompt.contains(&today),
            "Le prompt explorer doit contenir la date du jour ({today})"
        );
    }

    #[test]
    fn test_explorer_system_contains_platform() {
        let prompt = explorer_system();
        let os = std::env::consts::OS;
        assert!(
            prompt.contains(os),
            "Le prompt explorer doit contenir la plateforme OS ({os})"
        );
    }

    #[test]
    fn test_explorer_system_contains_truthfulness_rule() {
        let prompt = explorer_system();
        assert!(prompt.contains("Never invent files"));
    }

    #[test]
    fn test_coder_system_contains_constraints() {
        use crate::services::agent_local::subagent_prompts::CODER_SYSTEM_FOR_TEST;
        assert!(
            CODER_SYSTEM_FOR_TEST.contains("<constraints>"),
            "Le prompt coder doit contenir la balise <constraints>"
        );
    }

    #[test]
    fn test_coder_system_contains_security_and_truthfulness_rules() {
        use crate::services::agent_local::subagent_prompts::CODER_SYSTEM_FOR_TEST;
        assert!(CODER_SYSTEM_FOR_TEST.contains("Never invent files"));
        assert!(CODER_SYSTEM_FOR_TEST.contains("Fail closed on security errors"));
    }
}
