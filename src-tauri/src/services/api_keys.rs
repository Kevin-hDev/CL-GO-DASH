//! Gestion générique des clés API via le keystore OS natif (crate `keyring`).
//!
//! Principe de sécurité : aucune clé ne doit jamais être exposée au frontend.
//! Les commandes Tauri exposent set/delete/has/list/test, mais JAMAIS get.
//!
//! Pour éviter les popups Keychain répétées en dev (binaire non signé),
//! la liste des providers configurés est maintenue dans un fichier JSON.
//! Le Keychain n'est accédé que lors d'un appel API réel (get_key).

use keyring::Entry;
use reqwest::Client;
use std::time::Duration;
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "cl-go-dash";
const TEST_TIMEOUT: Duration = Duration::from_secs(10);

fn registry_path() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home directory");
    home.join(".local/share/cl-go-dash/configured-providers.json")
}

fn read_registry() -> Vec<String> {
    let path = registry_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn write_registry(ids: &[String]) -> Result<(), String> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(ids).map_err(|e| format!("json: {e}"))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json).map_err(|e| format!("write: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

fn add_to_registry(provider_id: &str) -> Result<(), String> {
    let mut ids = read_registry();
    if !ids.iter().any(|id| id == provider_id) {
        ids.push(provider_id.to_string());
        write_registry(&ids)?;
    }
    Ok(())
}

fn remove_from_registry(provider_id: &str) -> Result<(), String> {
    let mut ids = read_registry();
    let before = ids.len();
    ids.retain(|id| id != provider_id);
    if ids.len() != before {
        write_registry(&ids)?;
    }
    Ok(())
}

const KNOWN_PROVIDERS: &[&str] = &[
    "groq", "google", "mistral", "cerebras", "openrouter",
    "openai", "deepseek", "brave", "exa", "firecrawl", "serpapi", "google_cse",
];

/// Migration one-shot : si le registre n'existe pas encore, le crée
/// en scannant le Keychain. Déclenche les popups UNE DERNIÈRE FOIS.
pub fn migrate_registry_if_needed() {
    let path = registry_path();
    if path.exists() {
        return;
    }
    let mut found = Vec::new();
    for id in KNOWN_PROVIDERS {
        let Ok(entry) = Entry::new(KEYRING_SERVICE, id) else { continue };
        if entry.get_password().is_ok() {
            found.push(id.to_string());
        }
    }
    let _ = write_registry(&found);
    if !found.is_empty() {
        eprintln!("[api_keys] registre créé avec {} providers", found.len());
    }
}

/// Charge une clé API depuis le keystore OS.
pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    let entry =
        Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| format!("keyring entry: {e}"))?;
    let key = entry
        .get_password()
        .map_err(|e| format!("keyring get: {e}"))?;
    Ok(Zeroizing::new(key))
}

/// Stocke une clé API dans le keystore OS + registre local.
pub fn set_key(provider_id: &str, key: &str) -> Result<(), String> {
    let entry =
        Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| format!("keyring entry: {e}"))?;
    entry
        .set_password(key)
        .map_err(|e| format!("keyring set: {e}"))?;
    add_to_registry(provider_id)?;
    Ok(())
}

/// Supprime une clé API du keystore OS + registre local.
pub fn delete_key(provider_id: &str) -> Result<(), String> {
    let entry =
        Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| format!("keyring entry: {e}"))?;
    let _ = entry.delete_credential();
    remove_from_registry(provider_id)?;
    Ok(())
}

/// Vérifie si une clé est configurée (via registre local, pas de Keychain).
pub fn has_key(provider_id: &str) -> bool {
    read_registry().iter().any(|id| id == provider_id)
}

/// Retourne les provider_ids configurés (via registre local, pas de Keychain).
pub fn list_configured() -> Vec<String> {
    read_registry()
}

/// Teste qu'une clé API fonctionne via l'endpoint `/models` du provider.
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
        "google" => client
            .get("https://generativelanguage.googleapis.com/v1beta/models")
            .header("x-goog-api-key", key.as_str()),
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
            .header("Authorization", format!("Bearer {}", key.as_str())),
        "google_cse" => return Ok(()),
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
