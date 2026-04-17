//! Catalogue statique des providers LLM free-tier supportés.
//!
//! Chaque `ProviderSpec` contient tout ce dont on a besoin pour :
//! - Afficher la card dans le modal "Connecteurs API"
//! - Construire un `OpenAiCompatProvider` à partir d'un `provider_id` + clé

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str, // "llm"
    pub base_url: &'static str,
    pub models_endpoint: &'static str, // chemin relatif à base_url
    pub signup_url: &'static str,
    pub free_tier_label: &'static str,
    pub short_description: &'static str,
    /// Certains providers plafonnent la sortie si `max_tokens` absent
    /// (OpenAI/DeepSeek = 4k, Gemini = 8k). On force un max raisonnable.
    /// Groq/Mistral/Cerebras = unbounded → None (pas de plafond).
    /// **Groq** : surtout ne PAS mettre car leur free tier compte max_tokens dans le TPM budget.
    pub default_max_tokens: Option<u32>,
}

/// Retourne la spec d'un provider LLM par son id.
pub fn find(provider_id: &str) -> Option<&'static ProviderSpec> {
    LLM_PROVIDERS.iter().find(|p| p.id == provider_id)
}

pub const LLM_PROVIDERS: &[ProviderSpec] = &[
    ProviderSpec {
        id: "groq",
        display_name: "Groq",
        category: "llm",
        base_url: "https://api.groq.com/openai/v1",
        models_endpoint: "/models",
        signup_url: "https://console.groq.com/keys",
        free_tier_label: "14 400 req/day",
        short_description: "Inférence ultra-rapide Llama / Mixtral sur LPU custom.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "google",
        display_name: "Google Gemini",
        category: "llm",
        // Couche OpenAI-compat officielle de Google
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        models_endpoint: "/models",
        signup_url: "https://aistudio.google.com/app/apikey",
        free_tier_label: "1M tokens/min",
        short_description: "Gemini 2.5 Flash / 3.1 Pro — free tier permanent.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "mistral",
        display_name: "Mistral",
        category: "llm",
        base_url: "https://api.mistral.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://console.mistral.ai/api-keys",
        free_tier_label: "1B tokens/month",
        short_description: "Modèles open-weight français — Experiment plan gratuit.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "cerebras",
        display_name: "Cerebras",
        category: "llm",
        base_url: "https://api.cerebras.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://cloud.cerebras.ai/",
        free_tier_label: "1M tokens/day",
        short_description: "Llama 3.3 à 450 tok/s sur wafer-scale chip.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "openrouter",
        display_name: "OpenRouter",
        category: "llm",
        base_url: "https://openrouter.ai/api/v1",
        models_endpoint: "/models",
        signup_url: "https://openrouter.ai/settings/keys",
        free_tier_label: "30+ free models",
        short_description: "Accès à 300+ modèles via une seule clé API.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "openai",
        display_name: "OpenAI",
        category: "llm",
        base_url: "https://api.openai.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.openai.com/api-keys",
        free_tier_label: "$5 signup credits",
        short_description: "GPT-5.4, o-series reasoning models.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "deepseek",
        display_name: "DeepSeek",
        category: "llm",
        base_url: "https://api.deepseek.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.deepseek.com/api_keys",
        free_tier_label: "Low-cost ($0.30/Mtok)",
        short_description: "DeepSeek V4 / R1 — rapport qualité/prix imbattable.",
        default_max_tokens: None,
    },
];
