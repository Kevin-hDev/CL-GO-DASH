//! Client unifié pour les providers LLM OpenAI-compatibles.
//!
//! Tous nos providers retenus exposent `/chat/completions` avec auth Bearer.
//! Un seul client change juste `base_url` et charge la clé via `api_keys`.

use super::catalog::{self, ProviderSpec};
use super::openai_compat_models;
use super::openai_compat_parsing::{
    build_payload, map_error_status, parse_chat_response, parse_models_list,
};
use super::types::{ChatRequest, ChatResponse, LlmError, ModelInfo};
use crate::services::api_keys;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, LLM_BODY_LIMIT};

pub struct OpenAiCompatProvider {
    spec: &'static ProviderSpec,
    client: AuthenticatedClient,
}

pub fn ping_model(provider_id: &str) -> &'static str {
    openai_compat_models::ping_model(provider_id)
}

impl OpenAiCompatProvider {
    pub fn new(provider_id: &str) -> Result<Self, LlmError> {
        let spec = catalog::find(provider_id)
            .ok_or_else(|| LlmError::Provider(format!("provider inconnu : {}", provider_id)))?;
        let client = AuthenticatedClient::new(super::timeouts::request_timeout())
            .map_err(|_| network_error())?;
        Ok(Self { spec, client })
    }

    /// Appelle `/models` pour récupérer la liste des modèles disponibles.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        if let Some(models) = openai_compat_models::static_model_infos(self.spec.id) {
            return Ok(models);
        }
        let key = api_keys::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
        let url = format!("{}{}", self.spec.base_url, self.spec.models_endpoint);

        let request = self.client.get(&url).bearer_auth(&*key);
        let resp = self.send(request).await?;

        if !resp.status().is_success() {
            return Err(map_error_status(resp).await);
        }

        let body = read_json(resp).await?;
        parse_models_list(&body, self.spec.id)
    }

    /// Appelle `/chat/completions` en mode non-streaming.
    pub async fn chat_completion(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let key = api_keys::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
        let url = format!("{}/chat/completions", self.spec.base_url);
        let payload = build_payload(&req, false);

        let request = self.client.post(&url).bearer_auth(&*key).json(&payload);
        let resp = self.send(request).await?;

        if !resp.status().is_success() {
            return Err(map_error_status(resp).await);
        }

        let body = read_json(resp).await?;
        parse_chat_response(&body)
    }

    /// Test de connexion : appelle `/models` et vérifie HTTP 2xx.
    pub async fn test_connection(&self) -> Result<(), LlmError> {
        if openai_compat_models::has_static_models(self.spec.id) {
            return self.ping_chat().await;
        }
        self.list_models().await.map(|_| ())
    }

    async fn ping_chat(&self) -> Result<(), LlmError> {
        let key = api_keys::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
        let url = format!("{}/chat/completions", self.spec.base_url);
        let payload = serde_json::json!({
            "model": openai_compat_models::ping_model(self.spec.id),
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1,
        });
        let request = self.client.post(&url).bearer_auth(&*key).json(&payload);
        let resp = self.send(request).await?;
        if resp.status().is_success() {
            return Ok(());
        }
        Err(map_error_status(resp).await)
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response, LlmError> {
        self.client.send(request).await.map_err(|_| network_error())
    }
}

async fn read_json(response: reqwest::Response) -> Result<serde_json::Value, LlmError> {
    read_json_bounded(response, LLM_BODY_LIMIT)
        .await
        .map_err(|_| LlmError::Parse("réponse invalide".to_string()))
}

fn network_error() -> LlmError {
    LlmError::Network("requête refusée".to_string())
}

#[cfg(test)]
#[path = "openai_compat_http_tests.rs"]
mod tests;
