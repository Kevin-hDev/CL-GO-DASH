use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::{context_capsules, token_estimate};

const RESPONSE_RESERVE_PERCENT: u64 = 15;
const RESPONSE_RESERVE_MIN: u64 = 4_096;
const RESPONSE_RESERVE_MAX: u64 = 16_384;
const CHARS_PER_TOKEN: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBudgetReport {
    pub estimated_tokens: usize,
    pub max_input_tokens: Option<usize>,
    pub pruned_messages: usize,
}

pub fn prepare_for_request(
    messages: &mut Vec<ChatMessage>,
    context_window: u64,
) -> ContextBudgetReport {
    let estimated = token_estimate::estimate_tokens(messages);
    let Some(max_input) = max_input_tokens(context_window) else {
        return ContextBudgetReport {
            estimated_tokens: estimated,
            max_input_tokens: None,
            pruned_messages: 0,
        };
    };
    if estimated <= max_input {
        return ContextBudgetReport {
            estimated_tokens: estimated,
            max_input_tokens: Some(max_input),
            pruned_messages: 0,
        };
    }

    let original_len = messages.len();
    let capsule = context_capsules::recent_file_context_message(messages, context_window);
    let mut next: Vec<ChatMessage> = messages
        .iter()
        .filter(|m| m.role == "system")
        .cloned()
        .collect();
    context_capsules::insert_after_system(&mut next, capsule);

    let mut remaining_budget = max_input.saturating_sub(token_estimate::estimate_tokens(&next));
    let mut tail = Vec::new();
    for msg in messages.iter().rev().filter(|m| m.role != "system") {
        if remaining_budget == 0 {
            break;
        }
        let msg_tokens = token_estimate::estimate_tokens(std::slice::from_ref(msg));
        if msg_tokens <= remaining_budget {
            tail.push(msg.clone());
            remaining_budget -= msg_tokens;
        } else if tail.is_empty() {
            tail.push(trim_message(msg, remaining_budget));
            remaining_budget = 0;
        }
    }
    tail.reverse();
    next.extend(tail);
    *messages = next;

    ContextBudgetReport {
        estimated_tokens: token_estimate::estimate_tokens(messages),
        max_input_tokens: Some(max_input),
        pruned_messages: original_len.saturating_sub(messages.len()),
    }
}

pub fn max_input_tokens(context_window: u64) -> Option<usize> {
    if context_window == 0 {
        return None;
    }
    let reserve = response_reserve(context_window).min(context_window / 2);
    Some(context_window.saturating_sub(reserve) as usize)
}

fn response_reserve(context_window: u64) -> u64 {
    let target = context_window.saturating_mul(RESPONSE_RESERVE_PERCENT) / 100;
    target.clamp(RESPONSE_RESERVE_MIN, RESPONSE_RESERVE_MAX)
}

fn trim_message(msg: &ChatMessage, max_tokens: usize) -> ChatMessage {
    let max_chars = max_tokens.saturating_mul(CHARS_PER_TOKEN);
    let mut trimmed = msg.clone();
    trimmed.content = truncate_chars(&msg.content, max_chars);
    trimmed.images = None;
    trimmed.tool_calls = None;
    trimmed
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return "[message omitted: context budget exhausted]".to_string();
    }
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let kept: String = input.chars().take(max_chars).collect();
    format!("{kept}\n[message truncated for context budget]")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn unknown_context_does_not_prune() {
        let mut messages = vec![msg("user", &"x".repeat(100_000))];
        let report = prepare_for_request(&mut messages, 0);
        assert_eq!(report.max_input_tokens, None);
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn preserves_system_and_recent_tail() {
        let mut messages = vec![
            msg("system", "rules"),
            msg("user", &"a".repeat(80_000)),
            msg("assistant", "recent"),
        ];
        let report = prepare_for_request(&mut messages, 20_000);
        assert!(report.pruned_messages > 0);
        assert_eq!(messages[0].role, "system");
        assert!(messages.last().unwrap().content.contains("recent"));
    }
}
