use super::openai_compat_parsing::parse_models_list;
use serde_json::json;

#[test]
fn openrouter_models_use_supported_parameters_for_reasoning() {
    let body = json!({
        "data": [
            {
                "id": "provider/reasoning-model",
                "context_length": 128000,
                "pricing": { "prompt": "0", "completion": "0" },
                "supported_parameters": ["tools", "reasoning", "include_reasoning"]
            },
            {
                "id": "provider/plain-model",
                "pricing": { "prompt": "0.1", "completion": "0.2" },
                "supported_parameters": ["tools"]
            }
        ]
    });

    let models = parse_models_list(&body, "openrouter").unwrap();
    let reasoning = models
        .iter()
        .find(|m| m.id == "provider/reasoning-model")
        .unwrap();
    let plain = models
        .iter()
        .find(|m| m.id == "provider/plain-model")
        .unwrap();

    assert!(reasoning.supports_tools);
    assert!(reasoning.supports_thinking);
    assert_eq!(
        reasoning.reasoning_modes,
        ["off", "auto", "low", "medium", "high", "xhigh"]
    );
    assert!(!plain.supports_thinking);
    assert!(plain.reasoning_modes.is_empty());
}

#[test]
fn google_models_use_name_based_reasoning_modes() {
    let body = json!({
        "data": [
            {
                "id": "gemini-3.5-flash",
                "capabilities": { "completion_chat": true },
                "pricing": { "prompt": "0", "completion": "0" }
            },
            {
                "id": "gemini-2.5-flash",
                "capabilities": { "completion_chat": true },
                "pricing": { "prompt": "0", "completion": "0" }
            }
        ]
    });

    let models = parse_models_list(&body, "google").unwrap();
    let gemini_35 = models.iter().find(|m| m.id == "gemini-3.5-flash").unwrap();
    let gemini_25 = models.iter().find(|m| m.id == "gemini-2.5-flash").unwrap();

    assert_eq!(gemini_35.reasoning_modes, ["low", "medium", "high"]);
    assert_eq!(gemini_25.reasoning_modes, ["off", "low", "medium", "high"]);
}
