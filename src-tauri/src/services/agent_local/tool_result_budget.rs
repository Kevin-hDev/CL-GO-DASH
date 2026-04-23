use crate::services::agent_local::types_ollama::ChatMessage;

const MAX_TOTAL_RESULT_CHARS: usize = 100_000;
pub const CLEARED_PLACEHOLDER: &str =
    "[Résultat précédent tronqué — relancer le tool si nécessaire]";

/// Applique un budget sur les anciens tool results.
///
/// Si la somme totale des chars des messages `role="tool"` dépasse
/// `MAX_TOTAL_RESULT_CHARS`, les plus anciens sont remplacés par
/// `CLEARED_PLACEHOLDER` jusqu'à redescendre sous le budget.
/// Les 2 derniers tool results sont toujours préservés.
/// Les messages déjà tronqués (content == CLEARED_PLACEHOLDER) ne comptent pas.
pub fn apply_budget(messages: &mut [ChatMessage]) {
    // 1. Collecter les indices des messages role="tool" non déjà tronqués
    let tool_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == "tool" && m.content != CLEARED_PLACEHOLDER)
        .map(|(i, _)| i)
        .collect();

    // 2. Calculer le total des chars (Unicode — pas des octets)
    let total: usize = tool_indices
        .iter()
        .map(|&i| messages[i].content.chars().count())
        .sum();

    // 3. Si sous le budget, rien à faire
    if total <= MAX_TOTAL_RESULT_CHARS {
        return;
    }

    // 4. Tronquer les plus anciens en gardant les 2 derniers intacts
    let preserve_from = tool_indices.len().saturating_sub(2);
    let candidates = &tool_indices[..preserve_from];

    let mut remaining = total;
    for &idx in candidates {
        if remaining <= MAX_TOTAL_RESULT_CHARS {
            break;
        }
        remaining -= messages[idx].content.chars().count();
        messages[idx].content = CLEARED_PLACEHOLDER.to_string();
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    fn tool_msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: "tool".to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        }
    }

    fn user_msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: "user".to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn under_budget_no_change() {
        let mut msgs = vec![tool_msg("small"), tool_msg("also small")];
        apply_budget(&mut msgs);
        assert_eq!(msgs[0].content, "small");
        assert_eq!(msgs[1].content, "also small");
    }

    #[test]
    fn over_budget_truncates_oldest() {
        // 3 tool results : 60k + 60k + 60k = 180k > 100k
        let big = "x".repeat(60_000);
        let mut msgs = vec![
            tool_msg(&big),
            tool_msg(&big),
            tool_msg(&big),
        ];
        apply_budget(&mut msgs);
        // Le 1er (oldest) doit être tronqué
        assert_eq!(msgs[0].content, CLEARED_PLACEHOLDER);
        // Les 2 derniers préservés
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }

    #[test]
    fn keeps_last_two_tool_results() {
        // 4 tool results de 40k chacun = 160k > 100k
        let big = "y".repeat(40_000);
        let mut msgs = vec![
            tool_msg(&big),
            tool_msg(&big),
            tool_msg(&big),
            tool_msg(&big),
        ];
        apply_budget(&mut msgs);
        // Les 2 derniers doivent être préservés
        assert_eq!(msgs[2].content, big);
        assert_eq!(msgs[3].content, big);
    }

    #[test]
    fn non_tool_messages_ignored() {
        let big = "z".repeat(60_000);
        let mut msgs = vec![
            user_msg(&big),
            tool_msg(&big),
            tool_msg(&big),
        ];
        apply_budget(&mut msgs);
        // user message non touché
        assert_eq!(msgs[0].content, big);
        // 2 tool results sous 100k ensemble (60k+60k=120k > 100k mais 2 derniers préservés)
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }

    #[test]
    fn already_cleared_not_recounted() {
        let big = "w".repeat(60_000);
        let mut msgs = vec![
            ChatMessage {
                role: "tool".to_string(),
                content: CLEARED_PLACEHOLDER.to_string(),
                images: None,
                tool_calls: None,
                tool_name: None,
                tool_call_id: None,
            },
            tool_msg(&big),
            tool_msg(&big),
        ];
        // total actif = 120k > 100k, mais seulement 2 tool results actifs
        // → les 2 derniers sont préservés, rien n'est tronqué
        apply_budget(&mut msgs);
        assert_eq!(msgs[0].content, CLEARED_PLACEHOLDER);
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }
}
