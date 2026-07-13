const MAX_RAW_VALUE_LEN: usize = 8192;
const MAX_VAULT_ENTRIES: usize = 500;
const MAX_BATCH_ENTRIES: usize = 8;
const RAW_PREFIX: &str = "raw:";

pub fn set_key_raw(key_id: &str, value: &str) -> Result<(), String> {
    if key_id.is_empty() || key_id.len() > 128 {
        return Err("identifiant invalide".to_string());
    }
    if value.is_empty() || value.len() > 256 {
        return Err("valeur invalide".to_string());
    }
    let mut state = STATE.lock().map_err(|_| "erreur de stockage".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    if !current.keys.contains_key(key_id) && current.keys.len() >= MAX_VAULT_ENTRIES {
        return Err("limite du coffre atteinte".to_string());
    }
    current
        .keys
        .insert(key_id.to_string(), Zeroizing::new(value.to_string()));
    flush_vault(current)
}

pub fn delete_key_raw(key_id: &str) -> Result<(), String> {
    if key_id.is_empty() || key_id.len() > 128 {
        return Err("identifiant invalide".to_string());
    }
    let mut state = STATE.lock().map_err(|_| "erreur de stockage".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    current.keys.remove(key_id);
    flush_vault(current)
}

pub fn has_key(provider_id: &str) -> bool {
    let state = STATE.lock().ok();
    state
        .as_ref()
        .and_then(|current| current.as_ref())
        .map(|current| current.keys.contains_key(provider_id))
        .unwrap_or(false)
}

pub fn list_configured() -> Vec<String> {
    read_registry()
}

pub fn set_raw(key: &str, value: &str) -> Result<(), String> {
    set_raw_batch(&[(key, value)])
}

pub fn set_raw_batch(entries: &[(&str, &str)]) -> Result<(), String> {
    validate_raw_batch(entries)?;
    let prefixed_keys: Vec<String> = entries
        .iter()
        .map(|(key, _)| format!("{RAW_PREFIX}{key}"))
        .collect();
    let mut state = STATE.lock().map_err(|_| "erreur de stockage".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    let added = prefixed_keys
        .iter()
        .filter(|key| !current.keys.contains_key(*key))
        .count();
    if current.keys.len().saturating_add(added) > MAX_VAULT_ENTRIES {
        return Err("limite du coffre atteinte".to_string());
    }

    let mut candidate = ZeroizingMap(
        current
            .keys
            .iter()
            .map(|(key, value)| (key.clone(), value.as_str().to_string()))
            .collect(),
    );
    for (key, (_, value)) in prefixed_keys.iter().zip(entries) {
        candidate.0.insert(key.clone(), (*value).to_string());
    }
    vault::write_vault(&current.master_key, &candidate.0)?;
    for (key, (_, value)) in prefixed_keys.into_iter().zip(entries) {
        current
            .keys
            .insert(key, Zeroizing::new((*value).to_string()));
    }
    Ok(())
}

fn validate_raw_batch(entries: &[(&str, &str)]) -> Result<(), String> {
    if entries.is_empty() || entries.len() > MAX_BATCH_ENTRIES {
        return Err("lot de secrets invalide".to_string());
    }
    let mut unique = std::collections::HashSet::with_capacity(entries.len());
    for (key, value) in entries {
        if key.is_empty() || key.len() > 64 || !unique.insert(*key) {
            return Err("clé du coffre invalide".to_string());
        }
        if value.is_empty() || value.len() > MAX_RAW_VALUE_LEN {
            return Err("valeur du coffre invalide".to_string());
        }
    }
    Ok(())
}

pub fn get_raw(key: &str) -> Result<Zeroizing<String>, String> {
    let prefixed = format!("{RAW_PREFIX}{key}");
    let state = STATE.lock().map_err(|_| "erreur de stockage".to_string())?;
    let current = state
        .as_ref()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    current
        .keys
        .get(&prefixed)
        .cloned()
        .ok_or_else(|| "clé non trouvée".to_string())
}

pub fn delete_raw(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 64 {
        return Err("clé du coffre invalide".to_string());
    }
    let prefixed = format!("{RAW_PREFIX}{key}");
    let mut state = STATE.lock().map_err(|_| "erreur de stockage".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    current.keys.remove(&prefixed);
    flush_vault(current)
}
