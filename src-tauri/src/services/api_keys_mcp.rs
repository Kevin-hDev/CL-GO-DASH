const MCP_PREFIX: &str = "mcp_oauth_";
const MAX_CONNECTOR_ID: usize = 64;

fn validate_mcp_connector_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_CONNECTOR_ID {
        return Err("identifiant connecteur invalide".to_string());
    }
    if !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        return Err("identifiant connecteur invalide".to_string());
    }
    Ok(())
}

pub fn set_mcp_token(connector_id: &str, token_json: &str) -> Result<(), String> {
    validate_mcp_connector_id(connector_id)?;
    if token_json.is_empty() {
        return Err("token vide".to_string());
    }
    let key_id = format!("{MCP_PREFIX}{connector_id}");
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    if !s.keys.contains_key(&key_id) && s.keys.len() >= MAX_VAULT_ENTRIES {
        return Err("limite d'entrées vault atteinte".to_string());
    }
    s.keys.insert(key_id, Zeroizing::new(token_json.to_string()));
    flush_vault(s)
}

pub fn get_mcp_token(connector_id: &str) -> Result<Zeroizing<String>, String> {
    validate_mcp_connector_id(connector_id)?;
    let key_id = format!("{MCP_PREFIX}{connector_id}");
    let state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_ref().ok_or("vault not initialized")?;
    s.keys.get(&key_id).cloned().ok_or_else(|| "token non trouvé".to_string())
}

pub fn delete_mcp_token(connector_id: &str) -> Result<(), String> {
    validate_mcp_connector_id(connector_id)?;
    let key_id = format!("{MCP_PREFIX}{connector_id}");
    let mut state = STATE.lock().map_err(|e| format!("lock: {e}"))?;
    let s = state.as_mut().ok_or("vault not initialized")?;
    s.keys.remove(&key_id);
    flush_vault(s)
}

pub fn has_mcp_token(connector_id: &str) -> bool {
    if validate_mcp_connector_id(connector_id).is_err() {
        return false;
    }
    let key_id = format!("{MCP_PREFIX}{connector_id}");
    STATE
        .lock()
        .ok()
        .as_ref()
        .and_then(|s| s.as_ref())
        .map(|s| s.keys.contains_key(&key_id))
        .unwrap_or(false)
}
