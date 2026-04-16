//! Gestion générique des clés API via le keystore OS natif (crate `keyring`).
//!
//! Principe de sécurité : aucune clé ne doit jamais être exposée au frontend.
//! Les commandes Tauri exposent set/delete/has/list/test, mais JAMAIS get.
//! Le Rust charge la clé uniquement au moment de l'appel HTTPS et la libère ensuite.

use keyring::Entry;
use reqwest::Client;
use std::time::Duration;
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "cl-go-dash";
const TEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Liste des provider_ids connus. Sert à énumérer les clés configurées
/// (keyring n'expose pas d'API `list_entries`, on doit itérer sur une liste).
pub const KNOWN_PROVIDERS: &[&str] = &[
    // LLM providers free-tier
    "groq",
    "google",
    "mistral",
    "cerebras",
    "openrouter",
    "openai",
    "deepseek",
    // Search / scraping providers
    "brave",
    "exa",
    "firecrawl",
    "serpapi",
    "google_cse",
];

/// Charge une clé API depuis le keystore OS.
/// La clé est wrappée dans `Zeroizing<String>` — elle sera effacée de la mémoire
/// à la libération du scope.
pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    let entry =
        Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| format!("keyring entry: {e}"))?;
    let key = entry
        .get_password()
        .map_err(|e| format!("keyring get: {e}"))?;
    Ok(Zeroizing::new(key))
}

/// Stocke une clé API dans le keystore OS.
pub fn set_key(provider_id: &str, key: &str) -> Result<(), String> {
    let entry =
        Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| format!("keyring entry: {e}"))?;
    entry
        .set_password(key)
        .map_err(|e| format!("keyring set: {e}"))
}

/// Supprime une clé API du keystore OS.
pub fn delete_key(provider_id: &str) -> Result<(), String> {
    let entry =
        Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| format!("keyring entry: {e}"))?;
    entry
        .delete_credential()
        .map_err(|e| format!("keyring delete: {e}"))
}

/// Vérifie si une clé est configurée pour ce provider.
pub fn has_key(provider_id: &str) -> bool {
    let Ok(entry) = Entry::new(KEYRING_SERVICE, provider_id) else {
        return false;
    };
    entry.get_password().is_ok()
}

/// Retourne les provider_ids pour lesquels une clé est configurée.
pub fn list_configured() -> Vec<String> {
    KNOWN_PROVIDERS
        .iter()
        .filter(|id| has_key(id))
        .map(|s| s.to_string())
        .collect()
}

/// Teste qu'une clé API fonctionne en appelant l'endpoint `/models` (ou équivalent)
/// du provider. Retourne `Ok(())` si 2xx, sinon un message d'erreur human-friendly.
///
/// Note Phase 1 : mapping hardcodé ici. Sera déplacé dans `services/llm/catalog.rs`
/// en Phase 2 pour centraliser.
pub async fn test_key(provider_id: &str) -> Result<(), String> {
    let key = get_key(provider_id)?;
    let client = Client::builder()
        .timeout(TEST_TIMEOUT)
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let builder = match provider_id {
        "groq" => client
            .get("https://api.groq.com/openai/v1/models")
            .bearer_auth(&*key),
        "openai" => client
            .get("https://api.openai.com/v1/models")
            .bearer_auth(&*key),
        "openrouter" => client
            .get("https://openrouter.ai/api/v1/models")
            .bearer_auth(&*key),
        "cerebras" => client
            .get("https://api.cerebras.ai/v1/models")
            .bearer_auth(&*key),
        "mistral" => client
            .get("https://api.mistral.ai/v1/models")
            .bearer_auth(&*key),
        "deepseek" => client
            .get("https://api.deepseek.com/v1/models")
            .bearer_auth(&*key),
        "google" => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                &*key
            );
            client.get(url)
        }
        "brave" => client
            .get("https://api.search.brave.com/res/v1/web/search?q=test&count=1")
            .header("X-Subscription-Token", &*key),
        "exa" => client
            .post("https://api.exa.ai/search")
            .header("x-api-key", &*key)
            .json(&serde_json::json!({ "query": "test", "numResults": 1 })),
        "firecrawl" => client
            .get("https://api.firecrawl.dev/v2/team/credit-usage")
            .bearer_auth(&*key),
        "serpapi" => client
            .get("https://serpapi.com/account")
            .query(&[("api_key", key.as_str())]),
        "google_cse" => {
            // Nécessite aussi un CX (search engine ID) — pas testable seulement avec la clé
            // On accepte la clé sans test réel pour google_cse
            return Ok(());
        }
        other => return Err(format!("Provider inconnu : {}", other)),
    };

    let resp = builder
        .send()
        .await
        .map_err(|e| format!("network: {e}"))?;

    match resp.status().as_u16() {
        200..=299 => Ok(()),
        401 | 403 => Err("Clé API invalide ou non autorisée".to_string()),
        429 => Err("Rate limit atteint — clé valide mais quota dépassé".to_string()),
        status => Err(format!("Erreur serveur : HTTP {}", status)),
    }
}
