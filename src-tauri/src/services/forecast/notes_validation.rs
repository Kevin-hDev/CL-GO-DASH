use regex::Regex;
use std::sync::LazyLock;

const MAX_NOTE_BYTES: usize = 64 * 1024;
const MAX_TITLE_CHARS: usize = 120;
const MAX_FIELD_CHARS: usize = 80;
static SAFE_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-f0-9\-]{1,64}$").unwrap());
static SAFE_TYPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z_]{1,32}$").unwrap());

pub(super) fn id(id: &str, message: &str) -> Result<(), String> {
    if !SAFE_ID.is_match(id) {
        return Err(message.into());
    }
    Ok(())
}

pub(super) fn field(value: &str) -> Result<String, String> {
    clean_single_line(value, MAX_FIELD_CHARS, "Champ de note invalide")
}

pub(super) fn title(value: &str) -> Result<String, String> {
    clean_single_line(value, MAX_TITLE_CHARS, "Titre de note invalide")
}

pub(super) fn note_type(value: &str) -> Result<String, String> {
    let cleaned = value.trim();
    if !SAFE_TYPE.is_match(cleaned) {
        return Err("Type de note invalide".into());
    }
    Ok(cleaned.into())
}

pub(super) fn content(value: &str) -> Result<String, String> {
    if value.len() > MAX_NOTE_BYTES || value.contains('\0') {
        return Err("Contenu de note invalide".into());
    }
    Ok(value.trim().to_string())
}

fn clean_single_line(value: &str, max_chars: usize, error: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > max_chars || trimmed.contains('\0') {
        return Err(error.into());
    }
    Ok(trimmed.replace(['\n', '\r', '"'], ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbounded_or_unsafe_note_fields() {
        assert!(id("../note", "invalid").is_err());
        assert!(field("\n").is_err());
        assert!(title(&"a".repeat(MAX_FIELD_CHARS + 1)).is_ok());
        assert!(title(&"a".repeat(MAX_TITLE_CHARS + 1)).is_err());
        assert!(note_type("invalid type").is_err());
        assert!(content(&"a".repeat(MAX_NOTE_BYTES + 1)).is_err());
    }
}
