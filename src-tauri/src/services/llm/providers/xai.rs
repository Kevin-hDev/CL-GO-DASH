pub fn supports_tools(model: &str) -> bool {
    model.starts_with("grok-4")
        || model.starts_with("grok-3")
        || model.starts_with("grok-2")
        || model.starts_with("grok-code")
}

pub fn supports_thinking(model: &str) -> bool {
    model.contains("reasoning")
        || model.contains("multi-agent")
        || model.starts_with("grok-4.3")
        || model.starts_with("grok-3-mini")
}

pub fn supports_vision(model: &str) -> bool {
    model.contains("vision") || model.starts_with("grok-4")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools() {
        assert!(supports_tools("grok-4"));
        assert!(supports_tools("grok-4-1-fast-reasoning"));
        assert!(supports_tools("grok-3-beta"));
        assert!(supports_tools("grok-2-1212"));
        assert!(supports_tools("grok-code-fast"));
        assert!(!supports_tools("grok-beta"));
    }

    #[test]
    fn thinking() {
        assert!(supports_thinking("grok-4-1-fast-reasoning"));
        assert!(supports_thinking("grok-4-fast-reasoning"));
        assert!(supports_thinking("grok-4.20-multi-agent-beta-0309"));
        assert!(supports_thinking("grok-3-mini"));
        assert!(supports_thinking("grok-3-mini-fast-beta"));
        assert!(!supports_thinking("grok-4"));
        assert!(!supports_thinking("grok-3-beta"));
    }

    #[test]
    fn vision() {
        assert!(supports_vision("grok-4"));
        assert!(supports_vision("grok-4-latest"));
        assert!(supports_vision("grok-2-vision-1212"));
        assert!(supports_vision("grok-vision-beta"));
        assert!(!supports_vision("grok-3-beta"));
        assert!(!supports_vision("grok-3-mini"));
    }
}
