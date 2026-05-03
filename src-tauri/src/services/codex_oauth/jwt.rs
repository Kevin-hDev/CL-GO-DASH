use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

pub struct JwtClaims {
    pub account_id: String,
    pub email: Option<String>,
}

pub fn extract_claims(jwt: &str) -> Result<JwtClaims, String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err("JWT invalide : format incorrect".to_string());
    }
    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .or_else(|_| {
            use base64::engine::general_purpose::URL_SAFE;
            URL_SAFE.decode(parts[1])
        })
        .map_err(|_| "JWT invalide : base64 payload".to_string())?;
    let json: serde_json::Value =
        serde_json::from_slice(&payload).map_err(|_| "JWT invalide : JSON payload".to_string())?;

    let auth = &json["https://api.openai.com/auth"];
    let account_id = auth["chatgpt_account_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if account_id.is_empty() {
        return Err("JWT : chatgpt_account_id absent".to_string());
    }
    let profile = &json["https://api.openai.com/profile"];
    let email = profile["email"].as_str().map(String::from);

    Ok(JwtClaims { account_id, email })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jwt(payload: &serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(b"{}");
        let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload).unwrap());
        let sig = URL_SAFE_NO_PAD.encode(b"sig");
        format!("{header}.{body}.{sig}")
    }

    #[test]
    fn extracts_account_id_and_email() {
        let jwt = make_jwt(&serde_json::json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "acct_abc123",
            },
            "https://api.openai.com/profile": {
                "email": "test@example.com",
            },
        }));
        let claims = extract_claims(&jwt).unwrap();
        assert_eq!(claims.account_id, "acct_abc123");
        assert_eq!(claims.email.as_deref(), Some("test@example.com"));
    }

    #[test]
    fn rejects_missing_account_id() {
        let jwt = make_jwt(&serde_json::json!({"sub": "user123"}));
        assert!(extract_claims(&jwt).is_err());
    }

    #[test]
    fn rejects_invalid_format() {
        assert!(extract_claims("not-a-jwt").is_err());
        assert!(extract_claims("a.b").is_err());
    }
}
