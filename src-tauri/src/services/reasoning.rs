use crate::services::agent_local::types_ollama::OllamaThink;

pub fn sanitize_mode(mode: Option<String>) -> Option<String> {
    mode.filter(|value| {
        matches!(
            value.as_str(),
            "off" | "auto" | "low" | "medium" | "high" | "xhigh"
        )
    })
}

pub fn default_mode(provider: &str, model: &str) -> Option<String> {
    match provider {
        "codex-oauth" => Some("medium".to_string()),
        "ollama" if model.to_lowercase().contains("gpt-oss") => Some("medium".to_string()),
        _ => None,
    }
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
    if model.to_lowercase().contains("gpt-oss") {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
