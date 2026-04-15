use crate::services::agent_local::modelfile_parser::{
    merge_parameter, parse_modelfile, parse_param_value,
};
use crate::services::agent_local::types_ollama::{ModelInfo, OllamaModel};
use reqwest::Client;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:11434";
const TIMEOUT: Duration = Duration::from_secs(5);

pub struct OllamaClient {
    client: Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{BASE_URL}/api/tags"))
            .timeout(TIMEOUT)
            .send()
            .await
            .is_ok()
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let resp = self
            .client
            .get(format!("{BASE_URL}/api/tags"))
            .send()
            .await
            .map_err(|e| format!("Connexion Ollama impossible: {e}"))?;
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let models = json["models"]
            .as_array()
            .ok_or("Réponse invalide")?;

        let mut result = Vec::new();
        for m in models {
            let name = m["name"].as_str().unwrap_or_default().to_string();
            let info = self.show_model(&name).await.ok();
            result.push(build_model_from_tags(m, info));
        }
        Ok(result)
    }

    pub async fn get_modelfile(&self, name: &str) -> Result<String, String> {
        let info = self.show_model(name).await?;
        Ok(info.modelfile)
    }

    pub async fn update_modelfile(&self, name: &str, content: &str) -> Result<(), String> {
        let mut parsed = parse_modelfile(content);
        // /api/show renvoie FROM <blob sha256> ; Ollama rejette ce chemin comme nom de base.
        // Pour écraser un modèle existant, il faut forcer from = name du modèle.
        if needs_from_override(parsed.from.as_deref()) {
            parsed.from = Some(name.to_string());
        }
        let payload = parsed.to_api_payload(name);
        self.post_create(&payload).await
    }

    pub async fn update_system_prompt(&self, name: &str, system: &str) -> Result<(), String> {
        let current = self.get_modelfile(name).await?;
        let mut parsed = parse_modelfile(&current);
        parsed.system = Some(system.to_string());
        parsed.from = Some(name.to_string());
        parsed.license = None;
        let payload = parsed.to_api_payload(name);
        self.post_create(&payload).await
    }

    pub async fn update_parameters(
        &self,
        name: &str,
        entries: Vec<(String, String)>,
    ) -> Result<(), String> {
        let current = self.get_modelfile(name).await?;
        let mut parsed = parse_modelfile(&current);
        parsed.parameters.clear();
        for (k, v) in entries {
            let key = k.trim();
            let raw = v.trim();
            if key.is_empty() || raw.is_empty() {
                continue;
            }
            let value = parse_param_value(raw);
            merge_parameter(&mut parsed.parameters, key, value);
        }
        parsed.from = Some(name.to_string());
        // License inutile à renvoyer — Ollama la récupère depuis le modèle source via `from`.
        parsed.license = None;
        let payload = parsed.to_api_payload(name);
        self.post_create(&payload).await
    }

    async fn post_create(&self, payload: &serde_json::Value) -> Result<(), String> {
        let mut enriched = payload.clone();
        if let Some(obj) = enriched.as_object_mut() {
            obj.insert("stream".into(), serde_json::json!(false));
        }
        let resp = self
            .client
            .post(format!("{BASE_URL}/api/create"))
            .json(&enriched)
            .send()
            .await
            .map_err(|e| format!("Erreur /api/create: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Échec /api/create ({status}): {body}"));
        }
        Ok(())
    }

    pub async fn show_model(&self, name: &str) -> Result<ModelInfo, String> {
        let resp = self
            .client
            .post(format!("{BASE_URL}/api/show"))
            .json(&serde_json::json!({ "model": name }))
            .send()
            .await
            .map_err(|e| format!("Erreur show_model: {e}"))?;
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(parse_show_response(name, &json))
    }
}

fn needs_from_override(from: Option<&str>) -> bool {
    match from {
        None => true,
        Some(f) => f.starts_with('/') || f.contains("/blobs/sha256-"),
    }
}

fn build_model_from_tags(
    m: &serde_json::Value,
    info: Option<ModelInfo>,
) -> OllamaModel {
    let name = m["name"].as_str().unwrap_or_default().to_string();
    let details = &m["details"];
    OllamaModel {
        name,
        size: m["size"].as_u64().unwrap_or(0),
        family: info.as_ref().map_or_else(
            || s(details, "family"),
            |i| i.family.clone(),
        ),
        parameter_size: info.as_ref().map_or_else(
            || s(details, "parameter_size"),
            |i| i.parameter_size.clone(),
        ),
        quantization: info.as_ref().map_or_else(
            || s(details, "quantization_level"),
            |i| i.quantization.clone(),
        ),
        architecture: info.as_ref().map_or_else(String::new, |i| i.architecture.clone()),
        is_moe: info.as_ref().is_some_and(|i| i.is_moe),
        context_length: info.as_ref().map_or(0, |i| i.context_length),
        capabilities: info.map_or_else(
            || vec!["completion".to_string()],
            |i| i.capabilities,
        ),
    }
}

fn parse_show_response(name: &str, json: &serde_json::Value) -> ModelInfo {
    let details = &json["details"];
    let mi = &json["model_info"];
    let arch = mi["general.architecture"].as_str().unwrap_or("");

    ModelInfo {
        name: name.to_string(),
        modelfile: s(json, "modelfile"),
        parameters: s(json, "parameters"),
        template: s(json, "template"),
        family: s(details, "family"),
        parameter_size: s(details, "parameter_size"),
        quantization: s(details, "quantization_level"),
        architecture: arch.to_string(),
        is_moe: mi
            .get(format!("{arch}.expert_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        context_length: mi[format!("{arch}.context_length")]
            .as_u64()
            .unwrap_or(4096),
        capabilities: parse_capabilities(json),
        has_audio: json["capabilities"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("audio"))),
        license: s(json, "license"),
    }
}

fn parse_capabilities(json: &serde_json::Value) -> Vec<String> {
    json["capabilities"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_else(|| vec!["completion".to_string()])
}

fn s(v: &serde_json::Value, key: &str) -> String {
    v[key].as_str().unwrap_or("").to_string()
}
