//! Client unifié pour les providers LLM OpenAI-compatibles.
//!
//! Tous nos providers retenus exposent `/chat/completions` avec auth Bearer.
//! Un seul client change juste `base_url` et charge la clé via `api_keys`.

use super::catalog::{self, ProviderSpec};
use super::openai_compat_parsing::{build_payload, map_error_status, parse_chat_response, parse_models_list};
use super::types::{ChatRequest, ChatResponse, LlmError, ModelInfo};
use crate::services::api_keys;
use reqwest::Client;
use std::time::Duration;

fn static_models(provider_id: &str) -> Option<&'static [&'static str]> {
    match provider_id {
        "zai" => Some(&[
            "glm-5", "glm-5-code", "glm-4.7", "glm-4.6", "glm-4.5",
            "glm-4.5v", "glm-4.5-x", "glm-4.5-air", "glm-4.5-airx",
            "glm-4-32b-0414-128k", "glm-4.5-flash",
        ]),
        "xai" => Some(&[
            "grok-4", "grok-4-fast-reasoning", "grok-4-fast-non-reasoning",
            "grok-4.20-reasoning", "grok-4.20-non-reasoning",
            "grok-3", "grok-3-mini", "grok-3-fast",
            "grok-2-vision", "grok-code-fast",
        ]),
        _ => None,
    }
}

fn ping_model(provider_id: &str) -> &'static str {
    match provider_id {
        "zai" => "glm-4.5-flash",
        "xai" => "grok-3-mini",
        _ => "test",
    }
}

pub struct OpenAiCompatProvider {
    spec: &'static ProviderSpec,
    client: Client,
}

impl OpenAiCompatProvider {
    pub fn new(provider_id: &str) -> Result<Self, LlmError> {
        let spec = catalog::find(provider_id)
            .ok_or_else(|| LlmError::Provider(format!("provider inconnu : {}", provider_id)))?;
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| LlmError::Network(e.to_string()))?;
        Ok(Self { spec, client })
    }

    /// Appelle `/models` pour récupérer la liste des modèles disponibles.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        if let Some(models) = static_models(self.spec.id) {
            return Ok(models
                .iter()
                .map(|id| ModelInfo {
                    id: id.to_string(),
                    owned_by: None,
                    context_length: None,
                    supports_tools: false,
                    supports_vision: false,
                    supports_thinking: false,
                    is_free: false,
                })
                .collect());
        }
        let key = api_keys::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
        let url = format!("{}{}", self.spec.base_url, self.spec.models_endpoint);

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&*key)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(map_error_status(resp).await);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;
        parse_models_list(&body, self.spec.id)
    }

    /// Appelle `/chat/completions` en mode non-streaming.
    pub async fn chat_completion(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let key = api_keys::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
        let url = format!("{}/chat/completions", self.spec.base_url);
        let payload = build_payload(&req, false);

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&*key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(map_error_status(resp).await);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;
        parse_chat_response(&body)
    }

    /// Test de connexion : appelle `/models` et vérifie HTTP 2xx.
    pub async fn test_connection(&self) -> Result<(), LlmError> {
        if static_models(self.spec.id).is_some() {
            return self.ping_chat().await;
        }
        self.list_models().await.map(|_| ())
    }

    async fn ping_chat(&self) -> Result<(), LlmError> {
        let key = api_keys::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
        let url = format!("{}/chat/completions", self.spec.base_url);
        let payload = serde_json::json!({
            "model": ping_model(self.spec.id),
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1,
        });
        let resp = self.client.post(&url).bearer_auth(&*key).json(&payload)
            .send().await.map_err(|e| LlmError::Network(e.to_string()))?;
        if resp.status().is_success() {
            return Ok(());
        }
        Err(map_error_status(resp).await)
    }
}
