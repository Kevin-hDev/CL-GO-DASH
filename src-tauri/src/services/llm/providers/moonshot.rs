pub fn supports_tools(model: &str) -> bool {
    model.starts_with("kimi-k2")
        || model.starts_with("kimi-latest")
        || model.starts_with("kimi-for-coding")
}

pub fn supports_thinking(model: &str) -> bool {
    is_forced_thinking(model) || is_switchable_thinking(model)
}

pub fn is_forced_thinking(model: &str) -> bool {
    model.starts_with("kimi-k2.7-code")
        || model.contains("k2-thinking")
        || model.contains("thinking-preview")
        || model.starts_with("kimi-for-coding")
}

pub fn is_switchable_thinking(model: &str) -> bool {
    model.starts_with("kimi-k2.5") || model.starts_with("kimi-k2.6")
}

pub fn supports_vision(model: &str) -> bool {
    model.contains("k2.5")
        || model.contains("k2.6")
        || model.contains("k2.7")
        || model.starts_with("kimi-latest")
        || model.contains("thinking-preview")
        || model.contains("vision")
        || model.starts_with("kimi-for-coding")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools() {
        assert!(supports_tools("kimi-k2.6"));
        assert!(supports_tools("kimi-k2.7-code"));
        assert!(supports_tools("kimi-k2.7-code-highspeed"));
        assert!(supports_tools("kimi-k2.5"));
        assert!(supports_tools("kimi-k2-thinking"));
        assert!(supports_tools("kimi-latest"));
        assert!(supports_tools("kimi-latest-128k"));
        assert!(supports_tools("kimi-for-coding"));
        assert!(supports_tools("kimi-for-coding-highspeed"));
        assert!(!supports_tools("moonshot-v1-128k"));
    }

    #[test]
    fn thinking() {
        assert!(supports_thinking("kimi-k2-thinking"));
        assert!(supports_thinking("kimi-thinking-preview"));
        assert!(supports_thinking("kimi-k2.5"));
        assert!(supports_thinking("kimi-k2.6"));
        assert!(supports_thinking("kimi-k2.7-code"));
        assert!(supports_thinking("kimi-k2.7-code-highspeed"));
        assert!(supports_thinking("kimi-for-coding"));
        assert!(!supports_thinking("kimi-latest"));
    }

    #[test]
    fn vision() {
        assert!(supports_vision("kimi-k2.5"));
        assert!(supports_vision("kimi-k2.6"));
        assert!(supports_vision("kimi-k2.7-code"));
        assert!(supports_vision("kimi-k2.7-code-highspeed"));
        assert!(supports_vision("kimi-latest"));
        assert!(supports_vision("kimi-thinking-preview"));
        assert!(supports_vision("moonshot-v1-128k-vision-preview"));
        assert!(supports_vision("kimi-for-coding"));
        assert!(!supports_vision("kimi-k2-thinking"));
    }
}
