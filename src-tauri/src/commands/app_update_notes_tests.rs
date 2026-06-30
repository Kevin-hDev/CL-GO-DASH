use super::*;

fn valid_notes_json(version: &str) -> String {
    format!(
        r#"{{
  "{version}": {{
    "fr": ["Note française complète."],
    "en": ["Complete English note."],
    "es": ["Nota española completa."],
    "de": ["Vollständige deutsche Notiz."],
    "it": ["Nota italiana completa."],
    "zh": ["完整的中文说明。"],
    "ja": ["完全な日本語の説明です。"]
  }}
}}"#
    )
}

#[test]
fn parses_release_notes_for_matching_version() {
    let notes = parse_app_release_notes_json(valid_notes_json("0.9.4").as_bytes(), "0.9.4")
        .expect("notes");

    assert_eq!(notes["en"], vec!["Complete English note."]);
    assert_eq!(notes["fr"], vec!["Note française complète."]);
}

#[test]
fn accepts_v_prefixed_version_keys() {
    let notes = parse_app_release_notes_json(valid_notes_json("v0.9.4").as_bytes(), "0.9.4")
        .expect("notes");

    assert_eq!(notes["ja"], vec!["完全な日本語の説明です。"]);
}

#[test]
fn rejects_missing_locale() {
    let json = r#"{
      "0.9.4": {
        "en": ["Complete English note."]
      }
    }"#;

    assert!(parse_app_release_notes_json(json.as_bytes(), "0.9.4").is_none());
}

#[test]
fn rejects_overlong_notes_without_truncating() {
    let json = valid_notes_json("0.9.4").replace(
        "Complete English note.",
        &format!("{}.", "x".repeat(MAX_BULLET_CHARS + 1)),
    );

    assert!(parse_app_release_notes_json(json.as_bytes(), "0.9.4").is_none());
}

#[test]
fn rejects_incomplete_sentences() {
    let json = valid_notes_json("0.9.4").replace("Complete English note.", "Incomplete English note");

    assert!(parse_app_release_notes_json(json.as_bytes(), "0.9.4").is_none());
}

#[test]
fn rejects_large_payloads() {
    let bytes = vec![b' '; MAX_RELEASE_NOTES_BYTES + 1];

    assert!(parse_app_release_notes_json(&bytes, "0.9.4").is_none());
}
