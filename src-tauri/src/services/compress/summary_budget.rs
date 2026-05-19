pub const SMALL_CONTEXT_LIMIT: u64 = 64_000;
pub const LARGE_CONTEXT_LIMIT: u64 = 200_000;

const SMALL_SUMMARY_MAX: u64 = 8_000;
const MEDIUM_SUMMARY_MAX: u64 = 32_000;
const LARGE_SUMMARY_MAX: u64 = 50_000;
const SUMMARY_PERCENT: u64 = 15;
const EXTRA_LARGE_SUMMARY_PERCENT: u64 = 10;

pub fn summary_token_limit(context_window: u64) -> Option<u64> {
    if context_window == 0 {
        return None;
    }
    let pct = if context_window > 1_000_000 {
        EXTRA_LARGE_SUMMARY_PERCENT
    } else {
        SUMMARY_PERCENT
    };
    let target = context_window.saturating_mul(pct) / 100;
    let max = if context_window < SMALL_CONTEXT_LIMIT {
        SMALL_SUMMARY_MAX
    } else if context_window < LARGE_CONTEXT_LIMIT {
        MEDIUM_SUMMARY_MAX
    } else {
        LARGE_SUMMARY_MAX
    };
    Some(target.min(max).max(1_000))
}

pub fn summary_instruction(context_window: u64) -> Option<String> {
    summary_token_limit(context_window).map(|limit| {
        format!(
            "Keep the summary generous but bounded. Target at most {limit} tokens. \
             Preserve concrete files, decisions, errors, user constraints, current state, \
             and next steps. Do not drop important implementation details."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_50k_cap_for_large_contexts() {
        assert_eq!(summary_token_limit(400_000), Some(50_000));
        assert_eq!(summary_token_limit(1_000_000), Some(50_000));
    }

    #[test]
    fn keeps_unknown_context_unbounded() {
        assert_eq!(summary_token_limit(0), None);
        assert!(summary_instruction(0).is_none());
    }
}
