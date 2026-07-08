pub const CLAUDIATOR: &str = "Claudiator";
pub const GEMINITOR: &str = "Geminitor";
pub const CLAUDIATOR_COLOR: &str = "claudiator";
pub const GEMINITOR_COLOR: &str = "geminitor";

const MAX_NAME_CHARS: usize = 100;
const MAX_DESCRIPTION_CHARS: usize = 160;

pub fn default_name(subagent_type: &str) -> &'static str {
    match subagent_type {
        "coder" => CLAUDIATOR,
        _ => GEMINITOR,
    }
}

pub fn default_color_key(subagent_type: &str) -> &'static str {
    match subagent_type {
        "coder" => CLAUDIATOR_COLOR,
        _ => GEMINITOR_COLOR,
    }
}

pub fn clean_name(input: Option<&str>, subagent_type: &str) -> String {
    let fallback = default_name(subagent_type);
    bounded_non_empty(input, fallback, MAX_NAME_CHARS)
}

pub fn clean_description(input: Option<&str>, prompt: &str) -> String {
    let fallback = prompt
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Mission sous-agent");
    bounded_non_empty(input, fallback, MAX_DESCRIPTION_CHARS)
}

fn bounded_non_empty(input: Option<&str>, fallback: &str, max_chars: usize) -> String {
    let raw = input
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_subagent_type() {
        assert_eq!(default_name("coder"), "Claudiator");
        assert_eq!(default_color_key("coder"), "claudiator");
        assert_eq!(default_name("explorer"), "Geminitor");
        assert_eq!(default_color_key("explorer"), "geminitor");
    }
}
