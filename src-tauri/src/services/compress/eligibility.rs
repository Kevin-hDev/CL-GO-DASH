const MIN_NATIVE_CONTEXT: u64 = 131_072; // 128k tokens

pub fn is_model_eligible(native_context_length: u64) -> bool {
    native_context_length >= MIN_NATIVE_CONTEXT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_128k_eligible() {
        assert!(is_model_eligible(131_072));
    }

    #[test]
    fn model_1m_eligible() {
        assert!(is_model_eligible(1_048_576));
    }

    #[test]
    fn model_32k_not_eligible() {
        assert!(!is_model_eligible(32_768));
    }

    #[test]
    fn model_0_not_eligible() {
        assert!(!is_model_eligible(0));
    }

    #[test]
    fn model_just_under_128k_not_eligible() {
        assert!(!is_model_eligible(131_071));
    }
}
