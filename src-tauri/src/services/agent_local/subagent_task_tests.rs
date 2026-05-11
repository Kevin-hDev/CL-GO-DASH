#[cfg(test)]
mod tests {
    use crate::services::agent_local::types_ollama::ChatMessage;

    // Accès à la fonction privée via cfg(test) re-export dans subagent_task.rs
    use crate::services::agent_local::subagent_task::extract_summary_for_test;

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_summary_assistant() {
        let msgs = vec![msg("user", "bonjour"), msg("assistant", "réponse finale")];
        let summary = extract_summary_for_test(&msgs);
        assert_eq!(summary, "réponse finale");
    }

    #[test]
    fn test_extract_summary_tool_results() {
        let msgs = vec![msg("user", "bonjour"), msg("tool", "résultat outil")];
        let summary = extract_summary_for_test(&msgs);
        assert!(
            summary.contains("résultat outil"),
            "Les tool results doivent être utilisés quand il n'y a pas de message assistant"
        );
    }

    #[test]
    fn test_extract_summary_empty() {
        let msgs: Vec<ChatMessage> = vec![];
        let summary = extract_summary_for_test(&msgs);
        assert_eq!(summary, "Aucune réponse");
    }

    #[test]
    fn test_extract_summary_truncates() {
        let long_content = "x".repeat(5000);
        let msgs = vec![msg("tool", &long_content)];
        let summary = extract_summary_for_test(&msgs);
        // 2000 chars max + le préfixe "[Résultats d'outils]\n"
        let content_part = summary
            .strip_prefix("[Résultats d'outils]\n")
            .expect("Le préfixe doit être présent");
        assert!(
            content_part.len() <= 2000,
            "Le contenu tronqué doit faire au max 2000 chars, il en fait {}",
            content_part.len()
        );
    }
}
