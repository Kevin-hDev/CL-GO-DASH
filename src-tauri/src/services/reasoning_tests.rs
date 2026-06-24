use super::agent_local::types_ollama::OllamaThink;
use super::reasoning::*;

#[test]
fn codex_default_is_medium_and_no_off() {
    assert_eq!(codex_effort(None), "medium");
    assert_eq!(codex_effort(Some("off")), "medium");
    assert_eq!(codex_effort(Some("xhigh")), "xhigh");
}

#[test]
fn gpt_oss_uses_string_effort() {
    let think = ollama_think("gpt-oss:20b", Some("low"), false).unwrap();
    assert_eq!(think, OllamaThink::Level("low".to_string()));
}

#[test]
fn regular_ollama_uses_boolean_thinking() {
    let think = ollama_think("qwen3", Some("off"), true).unwrap();
    assert_eq!(think, OllamaThink::Bool(false));
}

#[test]
fn provider_specific_modes_are_distinct() {
    assert_eq!(
        supported_modes("groq", "openai/gpt-oss-20b", true),
        &["low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("groq", "qwen/qwen3-32b", true),
        &["off", "auto"]
    );
    assert_eq!(
        supported_modes("mistral", "magistral-medium-latest", true),
        &["auto"]
    );
    assert!(supported_modes("mistral", "mistral-small-2506", true).is_empty());
    assert_eq!(
        supported_modes("deepseek", "deepseek-v4-pro", true),
        &["off", "high", "xhigh"]
    );
    assert_eq!(
        supported_modes("google", "gemini-3.5-flash", true),
        &["low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("google", "gemini-2.5-flash", true),
        &["off", "low", "medium", "high"]
    );
    assert_eq!(
        supported_modes("zai", "glm-5.2", true),
        &["off", "auto", "low", "medium", "high", "xhigh"]
    );
    assert_eq!(
        supported_modes("moonshot", "kimi-k2.7-code", true),
        &["auto"]
    );
}

#[test]
fn xai_multi_agent_is_detected_as_thinking() {
    assert!(provider_model_supports_thinking(
        "xai",
        "grok-4.20-multi-agent-beta-0309"
    ));
}

#[test]
fn unsupported_model_clears_mode() {
    assert_eq!(
        normalize_for_model("ollama", "gemma4:latest", Some("auto"), false),
        None
    );
}
