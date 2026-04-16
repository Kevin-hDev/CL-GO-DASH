//! Wrapper léger sur `services::search` — conserve l'interface historique
//! pour l'agent tool `web_search` tout en déléguant au nouveau module search.

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use crate::services::search;

pub async fn web_search(query: &str) -> Result<Vec<SearchResult>, String> {
    search::run_search(query).await
}

/// Compat : ancienne commande `set_brave_api_key` → route vers `api_keys::set_key("brave", ...)`.
pub fn set_brave_key(key: &str) -> Result<(), String> {
    api_keys::set_key("brave", key)
}

/// Migration one-shot au démarrage : l'ancienne version stockait la clé Brave
/// sous le user `brave_api_key`. On la copie vers `brave` (nouveau nom canonique)
/// et on supprime l'ancienne entrée.
pub fn migrate_legacy_brave_key() {
    use keyring::Entry;
    const KEYRING_SERVICE: &str = "cl-go-dash";
    const LEGACY_USER: &str = "brave_api_key";

    // Si déjà migré, rien à faire
    if api_keys::has_key("brave") {
        return;
    }

    let Ok(legacy_entry) = Entry::new(KEYRING_SERVICE, LEGACY_USER) else {
        return;
    };
    let Ok(legacy_key) = legacy_entry.get_password() else {
        return;
    };

    if api_keys::set_key("brave", &legacy_key).is_ok() {
        let _ = legacy_entry.delete_credential();
        eprintln!("[migration] clé Brave migrée : brave_api_key → brave");
    }
}
