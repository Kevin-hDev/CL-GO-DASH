use crate::services::agent_local::types_ollama::OllamaThink;

pub fn sanitize_mode(mode: Option<String>) -> Option<String> {
    mode.filter(|value| {
        matches!(
            value.as_str(),
            "off" | "auto" | "low" | "medium" | "high" | "xhigh"
        )
    })
}

fn is_gpt_oss(model: &str) -> bool {
    model.to_lowercase().contains("gpt-oss")
}

fn is_xai_fixed_reasoning(model: &str) -> bool {
    let model = model.to_lowercase();
    model.contains("reasoning") || model.contains("multi-agent") || model.contains("4.20-reasoning")
}

fn lower(model: &str) -> String {
    model.to_lowercase()
}

pub fn supported_modes(
    provider: &str,
    model: &str,
    supports_thinking: bool,
) -> &'static [&'static str] {
    if !supports_thinking {
        return &[];
    }
    match provider {
        "codex-oauth" => &["low", "medium", "high", "xhigh"],
        "ollama" if is_gpt_oss(model) => &["low", "medium", "high"],
        "ollama" => &["off", "auto"],
        "openai" => &["off", "low", "medium", "high", "xhigh"],
        "openrouter" => &["off", "auto", "low", "medium", "high", "xhigh"],
        "groq" if crate::services::llm::providers::groq::is_gpt_oss_effort(&lower(model)) => {
            &["low", "medium", "high"]
        }
        "groq" if crate::services::llm::providers::groq::is_qwen_switchable(&lower(model)) => {
            &["off", "auto"]
        }
        "groq" => &["auto"],
        "deepseek" => &["off", "high", "xhigh"],
        "mistral"
            if crate::services::llm::providers::mistral::is_native_reasoning(&lower(model)) =>
        {
            &["auto"]
        }
        "mistral"
            if crate::services::llm::providers::mistral::is_adjustable_reasoning(&lower(model)) =>
        {
            &["off", "high"]
        }
        "mistral" => &[],
        "moonshot"
            if crate::services::llm::providers::moonshot::is_forced_thinking(&lower(model)) =>
        {
            &["auto"]
        }
        "moonshot" => &["off", "auto"],
        "xai" if is_xai_fixed_reasoning(model) => &["low", "medium", "high", "xhigh"],
        "xai" => &["off", "low", "medium", "high"],
        "zai" => &["off", "auto"],
        _ => &["off", "auto"],
    }
}

pub fn provider_model_supports_thinking(provider: &str, model: &str) -> bool {
    match provider {
        "codex-oauth" => true,
        "ollama" => is_gpt_oss(model),
        "openai" => {
            let model = model.to_lowercase();
            model.starts_with("o3") || model.starts_with("o4") || model.starts_with("gpt-5")
        }
        "deepseek" | "groq" | "google" | "openrouter" | "mistral" | "xai" | "moonshot" | "zai" => {
            crate::services::llm::tool_capable::supports_thinking(provider, model)
        }
        _ => false,
    }
}

pub fn normalize_for_model(
    provider: &str,
    model: &str,
    requested: Option<&str>,
    supports_thinking: bool,
) -> Option<String> {
    let modes = supported_modes(provider, model, supports_thinking);
    if modes.is_empty() {
        return None;
    }
    if let Some(mode) = requested.filter(|mode| modes.contains(mode)) {
        return Some(mode.to_string());
    }
    if modes.contains(&"medium") {
        return Some("medium".to_string());
    }
    if modes.contains(&"off") {
        return Some("off".to_string());
    }
    modes.first().map(|mode| mode.to_string())
}

pub fn default_mode(provider: &str, model: &str) -> Option<String> {
    normalize_for_model(
        provider,
        model,
        None,
        provider_model_supports_thinking(provider, model),
    )
}

pub fn enabled(mode: Option<&str>, fallback: bool) -> bool {
    match mode {
        Some("off") => false,
        Some(_) => true,
        None => fallback,
    }
}

pub fn codex_effort(mode: Option<&str>) -> String {
    match mode {
        Some("low" | "medium" | "high" | "xhigh") => mode.unwrap().to_string(),
        _ => "medium".to_string(),
    }
}

pub fn ollama_think(model: &str, mode: Option<&str>, fallback: bool) -> Option<OllamaThink> {
    if is_gpt_oss(model) {
        let effort = match mode {
            Some("low" | "medium" | "high") => mode.unwrap(),
            Some("xhigh") => "high",
            _ => "medium",
        };
        return Some(OllamaThink::Level(effort.to_string()));
    }
    Some(OllamaThink::Bool(enabled(mode, fallback)))
}

pub fn openai_effort(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("low") => Some("low"),
        Some("medium") | Some("auto") => Some("medium"),
        Some("high") => Some("high"),
        Some("xhigh") => Some("xhigh"),
        None => None,
        _ => None,
    }
}

pub fn simple_effort(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("low") => Some("low"),
        Some("medium") | Some("auto") => Some("medium"),
        Some("high") | Some("xhigh") => Some("high"),
        None => None,
        _ => None,
    }
}

pub fn openrouter_effort(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("low") => Some("low"),
        Some("medium") => Some("medium"),
        Some("high") => Some("high"),
        Some("xhigh") => Some("xhigh"),
        _ => None,
    }
}
