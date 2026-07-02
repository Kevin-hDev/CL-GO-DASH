//! Détection du bug Ollama "tool-call parser crash" (issue #16383).
//!
//! Quand Qwen3 génère un appel d'outil avec une légère erreur de formatage
//! (balises imbriquées dans le désordre, ex. `</parameter></function>`),
//! le parser d'Ollama est trop strict : il crash au lieu de tolérer la dérive.
//!
//! Deux manifestations selon le moment où Ollama détecte l'erreur :
//! - HTTP 500 avec body `{"error":"expected element type <function> but have <parameter>"}`
//!   si le crash survient avant ou pendant la génération
//! - Chunk NDJSON `{"error":"..."}` si le crash survient en plein stream
//!
//! Contournement : retenter la requête (le modèle générera probablement un
//! format correct au 2e essai, car le bug est intermittent).

/// Patterns d'erreur caractéristiques du parser tool-call trop strict.
/// On reste volontairement ciblé pour ne pas masquer d'autres erreurs 500.
const PARSER_ERROR_MARKERS: &[&str] = &[
    "expected element type",
    "<function>",
    "<parameter>",
];

/// Nombre maximum de retries pour ce bug précis.
/// Au-delà, on laisse l'erreur remonter (évite une boucle infinie si le
/// modèle produit systématiquement un format cassé).
pub const MAX_PARSER_RETRIES: u32 = 2;

/// Renvoie `true` si le texte correspond au crash du parser tool-call.
/// Insensible à la casse car Ollama peut formater le message différemment.
pub fn is_tool_parse_crash(text: &str) -> bool {
    let lower = text.to_lowercase();
    PARSER_ERROR_MARKERS.iter().all(|m| lower.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_exact_error_message() {
        let msg = r#"{"error":"expected element type <function> but have <parameter>"}"#;
        assert!(is_tool_parse_crash(msg));
    }

    #[test]
    fn detects_plain_text_error() {
        let msg = "expected element type <function> but have <parameter>";
        assert!(is_tool_parse_crash(msg));
    }

    #[test]
    fn does_not_match_unrelated_500() {
        assert!(!is_tool_parse_crash("internal server error"));
        assert!(!is_tool_parse_crash("model not found"));
        assert!(!is_tool_parse_crash("OOM: out of memory"));
    }

    #[test]
    fn does_not_match_partial_markers() {
        // Un seul marqueur ne suffit pas — il faut les 3 pour éviter les faux positifs
        assert!(!is_tool_parse_crash("expected element type <something>"));
        assert!(!is_tool_parse_crash("missing <function> definition"));
    }

    #[test]
    fn detects_case_insensitive() {
        let msg = "Expected Element Type <FUNCTION> but have <Parameter>";
        assert!(is_tool_parse_crash(msg));
    }
}
