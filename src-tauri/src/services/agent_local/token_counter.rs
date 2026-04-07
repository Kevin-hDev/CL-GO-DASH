/// Estime le nombre de tokens pour un texte donné.
///
/// Utilise tiktoken si un encodage connu existe pour la famille de modèle,
/// sinon fallback sur une approximation par ratio caractères/token.
pub fn estimate_tokens(text: &str, model_family: &str) -> usize {
    if let Some(count) = count_with_tiktoken(text, model_family) {
        return count;
    }
    approximate_tokens(text)
}

fn count_with_tiktoken(text: &str, model_family: &str) -> Option<usize> {
    let encoding_name = match model_family {
        f if f.contains("llama") => "llama3",
        f if f.contains("qwen") => "qwen2",
        f if f.contains("deepseek") => "deepseek-v3",
        f if f.contains("mistral") => "mistral",
        _ => return None,
    };
    tiktoken::encoding_for_model(encoding_name).map(|enc| enc.count(text))
}

fn approximate_tokens(text: &str) -> usize {
    let len = text.len() as f64;
    let ratio = detect_content_ratio(text);
    (len / ratio).ceil() as usize
}

fn detect_content_ratio(text: &str) -> f64 {
    let sample: String = text.chars().take(500).collect();
    let ascii_count = sample.chars().filter(|c| c.is_ascii()).count();
    let total = sample.len().max(1);
    let ascii_ratio = ascii_count as f64 / total as f64;

    let has_code_markers = sample.contains('{')
        || sample.contains("fn ")
        || sample.contains("function ")
        || sample.contains("import ");

    if has_code_markers {
        3.0 // code
    } else if ascii_ratio < 0.7 {
        2.5 // CJK ou accents lourds
    } else if ascii_ratio < 0.9 {
        3.5 // français
    } else {
        4.0 // anglais
    }
}
