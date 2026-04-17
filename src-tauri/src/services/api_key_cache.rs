//! Cache mémoire des clés API pour éviter les accès Keychain répétés.
//! Chaque clé est lue UNE FOIS depuis le Keychain au premier accès,
//! puis servie depuis le cache. Mise à jour sur set/delete.

use std::collections::HashMap;
use std::sync::Mutex;
use zeroize::Zeroizing;

use super::api_keys;

static CACHE: std::sync::LazyLock<Mutex<HashMap<String, Zeroizing<String>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Récupère la clé depuis le cache. Au premier accès, charge depuis le Keychain.
pub fn get_key(provider_id: &str) -> Result<Zeroizing<String>, String> {
    let mut cache = CACHE.lock().map_err(|e| format!("cache lock: {e}"))?;
    if let Some(key) = cache.get(provider_id) {
        return Ok(key.clone());
    }
    let key = api_keys::get_key(provider_id)?;
    cache.insert(provider_id.to_string(), key.clone());
    Ok(key)
}

/// Invalide le cache pour ce provider (appelé après set/delete).
pub fn invalidate(provider_id: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.remove(provider_id);
    }
}
