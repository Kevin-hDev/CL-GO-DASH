pub async fn test_key_raw(provider_id: &str, key: &str) -> Result<(), String> {
    validate::validate_key_input(provider_id, key)?;
    let client = Client::builder().timeout(Duration::from_secs(10)).build()
        .map_err(|e| format!("http: {e}"))?;
    if let Some(spec) = crate::services::llm::catalog::find(provider_id) {
        let resp = if !spec.models_endpoint.is_empty() {
            client.get(format!("{}{}", spec.base_url, spec.models_endpoint))
                .bearer_auth(key).send().await
        } else {
            let model = crate::services::llm::openai_compat::ping_model(provider_id);
            client.post(format!("{}/chat/completions", spec.base_url))
                .bearer_auth(key)
                .json(&serde_json::json!({
                    "model": model, "max_tokens": 1,
                    "messages": [{"role":"user","content":"hi"}]
                })).send().await
        };
        return check_status(resp.map_err(|e| format!("network: {e}"))?);
    }
    let resp = match provider_id {
        "brave" => client.get("https://api.search.brave.com/res/v1/web/search?q=test&count=1")
            .header("X-Subscription-Token", key),
        "exa" => client.post("https://api.exa.ai/search").header("x-api-key", key)
            .json(&serde_json::json!({"query":"test","numResults":1})),
        "firecrawl" => client.get("https://api.firecrawl.dev/v2/team/credit-usage")
            .bearer_auth(key),
        other => return Err(format!("Provider inconnu : {other}")),
    }.send().await.map_err(|e| format!("network: {e}"))?;
    check_status(resp)
}

fn check_status(resp: reqwest::Response) -> Result<(), String> {
    match resp.status().as_u16() {
        200..=299 => Ok(()),
        401 | 403 => Err("Clé API invalide ou non autorisée".into()),
        429 => Err("Rate limit — clé valide mais quota dépassé".into()),
        s => Err(format!("Erreur HTTP {s}")),
    }
}
