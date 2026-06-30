const MAX_SECTIONS: usize = 3;
const MAX_BULLETS: usize = 6;
const MAX_LINE_CHARS: usize = 180;
const MAX_TOTAL_CHARS: usize = 1200;

pub(crate) fn compact_release_notes(body: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut sections = 0usize;
    let mut bullets = 0usize;
    let mut section_allowed = true;

    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() || line == "---" || line.starts_with("<!--") {
            continue;
        }

        if let Some(title) = markdown_heading(line) {
            sections += 1;
            section_allowed = sections <= MAX_SECTIONS;
            if section_allowed {
                let clean = clean_text(title);
                if clean.chars().count() > MAX_LINE_CHARS {
                    return None;
                }
                out.push(format!("### {}", clean));
            }
            continue;
        }

        if let Some(item) = markdown_bullet(line) {
            if section_allowed && bullets < MAX_BULLETS {
                let clean = clean_text(item);
                if clean.chars().count() > MAX_LINE_CHARS {
                    return None;
                }
                bullets += 1;
                out.push(format!("- {}", clean));
            }
            continue;
        }

        if out.is_empty() {
            let clean = clean_text(line);
            if clean.chars().count() > MAX_LINE_CHARS {
                return None;
            }
            out.push(clean);
        }
    }

    let notes = out.join("\n").trim().to_string();
    if notes.is_empty() || notes.chars().count() > MAX_TOTAL_CHARS {
        None
    } else {
        Some(notes)
    }
}

fn markdown_heading(line: &str) -> Option<&str> {
    line.strip_prefix("### ")
        .or_else(|| line.strip_prefix("## "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn markdown_bullet(line: &str) -> Option<&str> {
    line.strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn clean_text(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .replace("**", "")
        .replace('`', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_short_headings_and_bullets() {
        let notes = compact_release_notes(
            r#"
### Features
- **Context details** — Added detailed context usage.
- Font size now uses pixels.

### Fixes
- Settings apply at startup.
"#,
        )
        .expect("notes");

        assert!(notes.contains("### Features"));
        assert!(notes.contains("- Context details — Added detailed context usage."));
        assert!(notes.contains("### Fixes"));
        assert!(notes.contains("- Settings apply at startup."));
    }

    #[test]
    fn returns_none_for_empty_notes() {
        assert_eq!(compact_release_notes("\n---\n<!-- hidden -->\n"), None);
    }

    #[test]
    fn caps_verbose_release_notes_without_truncating_sentences() {
        let body = (0..12)
            .map(|i| format!("- Item {i}"))
            .collect::<Vec<_>>()
            .join("\n");

        let notes = compact_release_notes(&body).expect("notes");

        assert_eq!(notes.lines().count(), MAX_BULLETS);
        assert!(!notes.contains('…'));
    }

    #[test]
    fn rejects_overlong_lines_instead_of_truncating() {
        let body = format!("- {}", "x".repeat(MAX_LINE_CHARS + 1));

        assert_eq!(compact_release_notes(&body), None);
    }
}
