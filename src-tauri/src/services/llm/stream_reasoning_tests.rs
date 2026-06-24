use super::stream_reasoning;
use serde_json::json;

fn payload(provider: &str, model: &str, mode: Option<&str>) -> serde_json::Value {
    let mut payload = json!({});
    stream_reasoning::apply(&mut payload, provider, model, mode != Some("off"), mode);
    payload
}

#[test]
fn groq_payloads_match_model_family() {
    assert_eq!(
        payload("groq", "qwen/qwen3-32b", Some("off"))["reasoning_effort"],
        "none"
    );
    assert_eq!(
        payload("groq", "qwen/qwen3-32b", Some("auto"))["reasoning_effort"],
        "default"
    );
    assert_eq!(
        payload("groq", "openai/gpt-oss-20b", Some("high"))["reasoning_effort"],
        "high"
    );
}

#[test]
fn deepseek_payload_uses_thinking_and_effort() {
    let high = payload("deepseek", "deepseek-v4-pro", Some("high"));
    assert_eq!(high["thinking"], json!({ "type": "enabled" }));
    assert_eq!(high["reasoning_effort"], "high");

    let max = payload("deepseek", "deepseek-v4-pro", Some("xhigh"));
    assert_eq!(max["reasoning_effort"], "max");

    let off = payload("deepseek", "deepseek-v4-pro", Some("off"));
    assert_eq!(off["thinking"], json!({ "type": "disabled" }));
}

#[test]
fn moonshot_switchable_can_disable_thinking() {
    let off = payload("moonshot", "kimi-k2.5", Some("off"));
    assert_eq!(off["thinking"], json!({ "type": "disabled" }));

    let auto = payload("moonshot", "kimi-k2.5", Some("auto"));
    assert!(auto.get("thinking").is_none());

    let forced = payload("moonshot", "kimi-k2-thinking", Some("auto"));
    assert!(forced.get("thinking").is_none());

    let k27 = payload("moonshot", "kimi-k2.7-code", Some("auto"));
    assert!(k27.get("thinking").is_none());
}

#[test]
fn zai_glm_52_uses_reasoning_effort() {
    let high = payload("zai", "glm-5.2", Some("high"));
    assert_eq!(high["thinking"], json!({ "type": "enabled" }));
    assert_eq!(high["reasoning_effort"], "high");

    let off = payload("zai", "glm-5.2", Some("off"));
    assert_eq!(off["thinking"], json!({ "type": "disabled" }));
    assert_eq!(off["reasoning_effort"], "none");
}

#[test]
fn google_gemini_35_uses_reasoning_effort() {
    assert_eq!(
        payload("google", "gemini-3.5-flash", Some("low"))["reasoning_effort"],
        "low"
    );
}

#[test]
fn mistral_adjustable_uses_reasoning_effort() {
    assert_eq!(
        payload("mistral", "mistral-small-latest", Some("off"))["reasoning_effort"],
        "none"
    );
    assert_eq!(
        payload("mistral", "mistral-small-latest", Some("high"))["reasoning_effort"],
        "high"
    );
}
