pub fn supports_tools(model: &str) -> bool {
    model.starts_with("kimi-k2") || model.starts_with("kimi-latest")
}

pub fn supports_thinking(model: &str) -> bool {
    model.contains("thinking")
}

pub fn supports_vision(model: &str) -> bool {
    model.contains("k2.5")
        || model.contains("k2.6")
        || model.contains("vision")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools() {
        assert!(supports_tools("kimi-k2.6"));
        assert!(supports_tools("kimi-k2.5"));
        assert!(supports_tools("kimi-k2-thinking"));
        assert!(supports_tools("kimi-latest"));
        assert!(supports_tools("kimi-latest-128k"));
        assert!(!supports_tools("moonshot-v1-128k"));
    }

    #[test]
    fn thinking() {
        assert!(supports_thinking("kimi-k2-thinking"));
        assert!(supports_thinking("kimi-thinking-preview"));
        assert!(!supports_thinking("kimi-k2.6"));
        assert!(!supports_thinking("kimi-latest"));
    }

    #[test]
    fn vision() {
        assert!(supports_vision("kimi-k2.5"));
        assert!(supports_vision("kimi-k2.6"));
        assert!(supports_vision("moonshot-v1-128k-vision-preview"));
        assert!(!supports_vision("kimi-k2-thinking"));
        assert!(!supports_vision("kimi-latest"));
    }
}
