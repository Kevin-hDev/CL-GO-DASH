//! Accumulateur de `tool_calls` arrivant en fragments SSE (format OpenAI-compat).
//!
//! **Quirks gérés** :
//! - OpenAI / Mistral / DeepSeek / Cerebras / OpenRouter : `arguments` arrivent en fragments
//!   JSON fragmentés. On concatène par `index`.
//! - Groq : envoie chaque `tool_call` **complet** en un seul chunk. Même algo fonctionne.
//! - Gemini OpenAI-compat : champ `index` absent — fallback via ordre d'arrivée des `id`.

use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ToolCallAcc {
    pub id: String,
    pub name: String,
    /// Arguments bruts concaténés. Parsés en JSON Value à la finalisation.
    pub arguments: String,
}

#[derive(Debug, Default)]
pub struct ToolCallAccumulator {
    /// Map par index (ou index synthétique pour Gemini).
    pub by_index: HashMap<usize, ToolCallAcc>,
    /// Ordre d'insertion (pour préserver l'ordre final).
    pub order: Vec<usize>,
    /// Compteur pour Gemini (index absent).
    pub synthetic_counter: usize,
}

impl ToolCallAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingère un `delta.tool_calls[]` JSON array.
    pub fn ingest(&mut self, tool_calls: &[Value]) {
        for tc in tool_calls {
            let index = tc["index"]
                .as_u64()
                .map(|v| v as usize)
                .unwrap_or_else(|| {
                    // Quirk Gemini : pas d'index → on synthétise un nouveau à chaque `id` non vu.
                    let id = tc["id"].as_str().unwrap_or("");
                    if !id.is_empty() {
                        // Nouvel id jamais vu → incrémenter
                        let existing = self.by_index.values().find(|a| a.id == id);
                        if existing.is_none() {
                            let i = self.synthetic_counter;
                            self.synthetic_counter += 1;
                            return i;
                        }
                    }
                    // Par défaut 0 (cas simple non-parallèle)
                    0
                });

            let entry = self.by_index.entry(index).or_insert_with(|| {
                self.order.push(index);
                ToolCallAcc::default()
            });

            if let Some(id) = tc["id"].as_str() {
                if !id.is_empty() && entry.id.is_empty() {
                    entry.id = id.to_string();
                }
            }
            if let Some(name) = tc["function"]["name"].as_str() {
                if !name.is_empty() && entry.name.is_empty() {
                    entry.name = name.to_string();
                }
            }
            if let Some(args_frag) = tc["function"]["arguments"].as_str() {
                entry.arguments.push_str(args_frag);
            }
        }
    }

    /// Finalise : retourne (tool_calls au format Ollama, IDs alignés).
    /// Parse chaque `arguments` string comme JSON Value.
    pub fn finalize(self) -> (Vec<(String, Value)>, Vec<String>) {
        // Tri par order d'apparition (préserve l'ordre des parallel calls).
        let mut ordered: Vec<usize> = self.order.clone();
        ordered.dedup();

        let mut tool_calls = Vec::with_capacity(ordered.len());
        let mut ids = Vec::with_capacity(ordered.len());

        for idx in ordered {
            let Some(acc) = self.by_index.get(&idx) else {
                continue;
            };
            if acc.name.is_empty() {
                continue;
            }
            let args_value: Value = if acc.arguments.trim().is_empty() {
                Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str(&acc.arguments).unwrap_or_else(|_| {
                    // Argument non-parseable → on passe le string brut pour debug.
                    Value::String(acc.arguments.clone())
                })
            };
            tool_calls.push((acc.name.clone(), args_value));
            ids.push(acc.id.clone());
        }
        (tool_calls, ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accumulates_fragmented_arguments_openai_style() {
        // Cas OpenAI/Mistral/DeepSeek : arguments fragmentés sur plusieurs chunks.
        let mut acc = ToolCallAccumulator::new();
        acc.ingest(&[json!({
            "index": 0, "id": "call_1", "type": "function",
            "function": { "name": "web_search", "arguments": "" }
        })]);
        acc.ingest(&[json!({
            "index": 0, "function": { "arguments": "{\"query\":" }
        })]);
        acc.ingest(&[json!({
            "index": 0, "function": { "arguments": " \"rust tauri 2\"}" }
        })]);
        let (calls, ids) = acc.finalize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "web_search");
        assert_eq!(calls[0].1["query"], "rust tauri 2");
        assert_eq!(ids[0], "call_1");
    }

    #[test]
    fn accumulates_complete_tool_call_groq_style() {
        // Cas Groq : tool_call complet en un seul chunk.
        let mut acc = ToolCallAccumulator::new();
        acc.ingest(&[json!({
            "index": 0, "id": "call_x", "type": "function",
            "function": {
                "name": "read_file",
                "arguments": "{\"path\": \"/tmp/x\"}"
            }
        })]);
        let (calls, _) = acc.finalize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1["path"], "/tmp/x");
    }

    #[test]
    fn accumulates_parallel_tool_calls_by_index() {
        // Deux tools en parallèle avec index différents.
        let mut acc = ToolCallAccumulator::new();
        acc.ingest(&[
            json!({
                "index": 0, "id": "a", "type": "function",
                "function": { "name": "f1", "arguments": "{\"x\":1}" }
            }),
            json!({
                "index": 1, "id": "b", "type": "function",
                "function": { "name": "f2", "arguments": "{\"y\":2}" }
            }),
        ]);
        let (calls, ids) = acc.finalize();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "f1");
        assert_eq!(calls[1].0, "f2");
        assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn tolerates_gemini_missing_index() {
        // Quirk Gemini : index absent. On synthétise un index par id.
        let mut acc = ToolCallAccumulator::new();
        acc.ingest(&[json!({
            "id": "0", "type": "function",
            "function": { "name": "g1", "arguments": "{\"q\":\"x\"}" }
        })]);
        let (calls, _) = acc.finalize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "g1");
    }
}
