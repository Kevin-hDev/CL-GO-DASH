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

fn compute_signature(tool_calls: &[(String, serde_json::Value)]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for (name, args) in tool_calls {
        name.hash(&mut hasher);
        args.to_string().hash(&mut hasher);
    }
    hasher.finish()
}
