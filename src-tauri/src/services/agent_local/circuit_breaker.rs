use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_CONSECUTIVE_IDENTICAL: usize = 3;

pub struct CircuitBreaker {
    last_signature: Option<u64>,
    consecutive_count: usize,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self { last_signature: None, consecutive_count: 0 }
    }

    /// Vérifie les tool_calls. Retourne Err si boucle détectée.
    pub fn check(&mut self, tool_calls: &[(String, serde_json::Value)]) -> Result<(), String> {
        let sig = compute_signature(tool_calls);
        match self.last_signature {
            Some(prev) if prev == sig => {
                self.consecutive_count += 1;
                if self.consecutive_count >= MAX_CONSECUTIVE_IDENTICAL {
                    return Err(format!(
                        "Circuit breaker : {} appels identiques consécutifs détectés. Boucle probable, arrêt.",
                        self.consecutive_count
                    ));
                }
            }
            _ => {
                self.last_signature = Some(sig);
                self.consecutive_count = 1;
            }
        }
        Ok(())
    }
}

fn normalize_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut pairs: Vec<_> = map.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            let entries: Vec<String> = pairs
                .into_iter()
                .map(|(k, v)| format!("{}:{}", k, normalize_json(v)))
                .collect();
            format!("{{{}}}", entries.join(","))
        }
        serde_json::Value::Array(arr) => {
            let entries: Vec<String> = arr.iter().map(normalize_json).collect();
            format!("[{}]", entries.join(","))
        }
        _ => value.to_string(),
    }
}

fn compute_signature(tool_calls: &[(String, serde_json::Value)]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for (name, args) in tool_calls {
        name.hash(&mut hasher);
        normalize_json(args).hash(&mut hasher);
    }
    hasher.finish()
}
