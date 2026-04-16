//! Détection de la capacité tool-use (function calling) d'un modèle.
//!
//! Approche par patterns hardcodés, plus évolutive que les noms exacts.
//! À mettre à jour quand de nouveaux modèles sortent (env. tous les 2-3 mois).
//!
//! Pour Ollama : ne pas utiliser cette fonction — l'API `/api/show` expose
//! dynamiquement le flag `capabilities.tools`.
//!
//! Pour OpenRouter : l'API `/models` expose un champ `supports_tools` par modèle.
//! Cette fonction retourne `true` en permissif, on se fiera au flag API côté UI.

pub fn supports_tools(provider_id: &str, model_id: &str) -> bool {
    let model = model_id.to_lowercase();
    match provider_id {
        "groq" => {
            model.starts_with("llama-3.3")
                || model.starts_with("llama-3.1-70b")
                || model.starts_with("llama-3.1-8b")
                || model.starts_with("llama-4")
                || model.starts_with("mixtral-8x")
                || model.starts_with("deepseek")
        }
        "google" => {
            let has_gemini = model.contains("gemini");
            let is_pro = model.contains("pro");
            let is_flash_full = model.contains("flash") && !model.contains("flash-lite");
            has_gemini && (is_pro || is_flash_full)
        }
        "mistral" => {
            model.starts_with("mistral-large")
                || model.starts_with("mistral-medium")
                || model.starts_with("mistral-small-3")
                || model.starts_with("codestral")
                || model.starts_with("open-mistral-nemo")
                || model.starts_with("ministral")
        }
        "cerebras" => {
            model.starts_with("llama-3.1")
                || model.starts_with("llama-3.3")
                || model.starts_with("llama-4")
        }
        // OpenRouter : permissif, l'UI filtrera via flag supports_tools de l'API /models
        "openrouter" => true,
        "openai" => {
            model.starts_with("gpt-4")
                || model.starts_with("gpt-5")
                || model.starts_with("o3")
                || model.starts_with("o4-mini")
        }
        "deepseek" => model.starts_with("deepseek-chat") || model.starts_with("deepseek-v"),
        // Ollama : utilise la détection dynamique via /api/show ailleurs
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groq_tool_capable() {
        assert!(supports_tools("groq", "llama-3.3-70b-versatile"));
        assert!(supports_tools("groq", "llama-4-scout-17b-16e-instruct"));
        assert!(!supports_tools("groq", "whisper-large-v3"));
    }

    #[test]
    fn gemini_tool_capable() {
        assert!(supports_tools("google", "gemini-2.5-pro"));
        assert!(supports_tools("google", "gemini-3.1-pro"));
        assert!(supports_tools("google", "gemini-2.5-flash"));
        assert!(!supports_tools("google", "gemini-2.5-flash-lite"));
        assert!(!supports_tools("google", "text-embedding-004"));
    }

    #[test]
    fn mistral_tool_capable() {
        assert!(supports_tools("mistral", "mistral-large-latest"));
        assert!(supports_tools("mistral", "mistral-small-3-24b"));
        assert!(supports_tools("mistral", "codestral-latest"));
        assert!(!supports_tools("mistral", "mistral-embed"));
    }

    #[test]
    fn openai_tool_capable() {
        assert!(supports_tools("openai", "gpt-4o"));
        assert!(supports_tools("openai", "gpt-5.4"));
        assert!(supports_tools("openai", "o4-mini"));
        assert!(!supports_tools("openai", "text-embedding-3-small"));
    }

    #[test]
    fn unknown_provider_returns_false() {
        assert!(!supports_tools("unknown", "any-model"));
    }
}
