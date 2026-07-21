use serde_json::Value;

pub fn validate(tool: &str, args: &Value, definition: &Value) -> Result<Value, String> {
    let object = args
        .as_object()
        .ok_or_else(|| "les arguments doivent être un objet JSON".to_string())?;
    let parameters = definition
        .pointer("/function/parameters")
        .and_then(Value::as_object)
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let properties = parameters
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let required = parameters
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;

    for name in required.iter().filter_map(Value::as_str) {
        if !matches!(object.get(name), Some(value) if !value.is_null()) {
            return Err(format!("paramètre '{name}' requis"));
        }
    }

    let mut cleaned = serde_json::Map::with_capacity(properties.len());
    for (name, value) in object {
        let Some(property) = properties.get(name) else {
            eprintln!("[tool-validate] argument inconnu ignoré : {tool}.{name}");
            continue;
        };
        if !value.is_null() {
            validate_type(name, value, property)?;
        }
        cleaned.insert(name.clone(), value.clone());
    }
    Ok(Value::Object(cleaned))
}

fn validate_type(name: &str, value: &Value, property: &Value) -> Result<(), String> {
    let expected = property["type"]
        .as_str()
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let valid = match expected {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "boolean" => value.is_boolean(),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(format!("'{name}' doit être de type {expected}"))
    }
}
