use super::stream_convert::messages_to_openai;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::route::{self, LlmRoute, RouteError};
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
pub struct RequestConfig<'a> {
    pub provider_id: &'a str,
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub tools: &'a [serde_json::Value],
    pub think: bool,
    pub reasoning_mode: Option<&'a str>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug)]
pub enum RequestError {
    Fatal(String),
    RetryWithoutTools(String),
    RetryWithoutImages(String),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal(s) | Self::RetryWithoutTools(s) | Self::RetryWithoutImages(s) => {
                f.write_str(s)
            }
        }
    }
}

async fn send_json_request(
    client: &AuthenticatedClient,
    route: &LlmRoute,
    url: &str,
    payload: &serde_json::Value,
) -> Result<reqwest::Response, RequestError> {
    route
        .send_authenticated(client, |token, headers| {
            client
                .post(url)
                .headers(headers)
                .bearer_auth(token)
                .json(payload)
        })
        .await
        .map_err(|error| match error {
            RouteError::Unauthorized => RequestError::Fatal("Connexion requise".into()),
            RouteError::Network => {
                RequestError::Fatal("Connexion au fournisseur impossible".into())
            }
        })
}

async fn read_provider_error(response: reqwest::Response) -> zeroize::Zeroizing<String> {
    match read_bounded(response, PROVIDER_ERROR_LIMIT).await {
        Ok(bytes) => zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => zeroize::Zeroizing::new(String::new()),
    }
}

pub async fn post_chat_request(cfg: &RequestConfig<'_>) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout(cfg, super::timeouts::request_timeout()).await
}

pub async fn post_chat_request_with_timeout(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
) -> Result<reqwest::Response, RequestError> {
    if cfg.model.len() > 128 {
        return Err(RequestError::Fatal("nom de modèle trop long".into()));
    }
    let route = route::resolve(cfg.provider_id)
        .ok_or_else(|| RequestError::Fatal("Fournisseur inconnu".to_string()))?;
    let url = format!("{}/chat/completions", route.base_url);
    let payload = build_chat_payload(cfg, &route);

    let client = AuthenticatedClient::new(timeout)
        .map_err(|_| RequestError::Fatal("Connexion au fournisseur impossible".into()))?;
    let resp = send_json_request(&client, &route, &url, &payload).await?;

    if !route.is_oauth() && matches!(route.chat_provider_id, "groq" | "xai") {
        super::quota::update_ratelimit_headers(route.chat_provider_id, resp.headers());
    }

    let status = resp.status();
    if !status.is_success() {
        let body = read_provider_error(resp).await;
        eprintln!(
            "[llm stream] HTTP {} — {}",
            status,
            super::sanitize_log_body(&body)
        );
        return Err(classify_error(
            status.as_u16(),
            &body,
            route.display_name,
            route.is_oauth(),
        ));
    }
    Ok(resp)
}

fn build_chat_payload(cfg: &RequestConfig<'_>, route: &LlmRoute) -> serde_json::Value {
    let provider_id = route.canonical_provider_id;
    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": messages_to_openai(cfg.messages, provider_id),
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(max) = cfg.max_tokens.or(route.default_max_tokens) {
        let field = if matches!(provider_id, "openai" | "openrouter")
            && super::providers::openai::is_gpt_56(cfg.model)
        {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        payload[field] = max.into();
    }
    super::stream_reasoning::apply(
        &mut payload,
        provider_id,
        cfg.model,
        cfg.think,
        cfg.reasoning_mode,
    );
    if !cfg.tools.is_empty() {
        let tools = super::tool_schema::tools_for_provider(provider_id, cfg.model, cfg.tools);
        payload["tools"] = serde_json::Value::Array(tools);
        payload["tool_choice"] = "auto".into();
        if provider_id == "zai" {
            payload["tool_stream"] = true.into();
        }
    }
    if provider_id == "openrouter" {
        payload["provider"] = serde_json::json!({
            "require_parameters": true,
            "allow_fallbacks": true,
        });
    }
    payload
}

fn classify_error(status: u16, body: &str, provider_name: &str, oauth: bool) -> RequestError {
    match status {
        401 | 403 if oauth => RequestError::Fatal("Connexion OAuth requise".into()),
        401 | 403 => RequestError::Fatal("Clé API invalide ou non autorisée".into()),
        413 => RequestError::Fatal("Requête trop volumineuse (limite TPM dépassée)".into()),
        429 => RequestError::Fatal("Rate limit atteint, réessaie plus tard".into()),
        400 if body.contains("Developer instruction") || body.contains("system_instruction") => {
            RequestError::Fatal("Ce modèle ne supporte pas les instructions système via ce provider. Essaie un autre modèle.".into())
        }
        400 if body.contains("must be a string") => {
            RequestError::RetryWithoutImages("Format image non supporté par ce provider".into())
        }
        404 if body.contains("tool use") || body.contains("tool_use") => {
            RequestError::RetryWithoutTools("Aucun endpoint ne supporte les tools pour ce modèle".into())
        }
        404 if body.contains("image") => {
            RequestError::RetryWithoutImages("Aucun endpoint ne supporte les images pour ce modèle".into())
        }
        400 if body.contains("image") => {
            RequestError::RetryWithoutImages("Ce modèle ne supporte pas les images".into())
        }
        _ => RequestError::Fatal(format!("{provider_name} HTTP {status}")),
    }
}

#[cfg(test)]
#[path = "stream_http_tests.rs"]
mod tests;
