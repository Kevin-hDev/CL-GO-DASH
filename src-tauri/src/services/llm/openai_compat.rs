//! Client unifié pour les providers LLM OpenAI-compatibles.
//!
//! Tous nos providers retenus exposent `/chat/completions` avec auth Bearer.
//! Un seul client change juste `base_url` et charge la clé via `api_keys`.

use super::catalog::{self, ProviderSpec};
use super::openai_compat_parsing::{build_payload, map_error_status, parse_chat_response, parse_models_list};
use super::types::{ChatRequest, ChatResponse, LlmError, ModelInfo};
use crate::services::api_key_cache;
use reqwest::Client;
use std::time::Duration;

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

    pub fn provider_id(&self) -> &str {
        self.spec.id
    }

    /// Appelle `/models` pour récupérer la liste des modèles disponibles.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        let key = api_key_cache::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
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
    /// Le streaming SSE sera ajouté en Phase 6 (méthode `chat_completion_stream`).
    pub async fn chat_completion(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let key = api_key_cache::get_key(self.spec.id).map_err(|_| LlmError::Unauthorized)?;
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
        self.list_models().await.map(|_| ())
    }
}
