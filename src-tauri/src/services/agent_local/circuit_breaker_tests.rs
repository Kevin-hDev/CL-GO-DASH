#[cfg(test)]
mod tests {
    use crate::services::agent_local::circuit_breaker::CircuitBreaker;
    use serde_json::json;

    fn call(name: &str, arg: i64) -> Vec<(String, serde_json::Value)> {
        vec![(name.to_string(), json!({ "n": arg }))]
    }

    #[test]
    fn allows_different_calls() {
        let mut breaker = CircuitBreaker::new();
        for i in 0..10 {
            let calls = call("bash", i);
            assert!(
                breaker.check(&calls).is_ok(),
                "appel #{i} devrait être autorisé"
            );
        }
    }

    #[test]
    fn trips_on_identical_calls() {
        let mut breaker = CircuitBreaker::new();
        let calls = call("bash", 42);

        // 1er appel : OK
        assert!(breaker.check(&calls).is_ok());
        // 2ème appel identique : OK (seuil = 3, count = 2)
        assert!(breaker.check(&calls).is_ok());
        // 3ème appel identique : ERREUR (count atteint MAX_CONSECUTIVE_IDENTICAL)
        let result = breaker.check(&calls);
        assert!(
            result.is_err(),
            "le 3ème appel identique consécutif devrait déclencher le circuit breaker"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("Circuit breaker"),
            "message inattendu : {msg}"
        );
    }

    #[test]
    fn trips_on_reordered_json_keys() {
        // Deux tool_calls identiques en valeur mais avec clés JSON dans un ordre différent
        // doivent produire la même signature → détecter la boucle
        let mut breaker = CircuitBreaker::new();

        let call_ab = vec![("write_file".to_string(), json!({ "path": "x", "content": "y" }))];
        let call_ba = vec![("write_file".to_string(), json!({ "content": "y", "path": "x" }))];

        // 1er appel (ab) : OK
        assert!(breaker.check(&call_ab).is_ok());
        // 2ème appel (ba, clés inversées) : même signature normalisée → compteur = 2, OK
        assert!(breaker.check(&call_ba).is_ok());
        // 3ème appel (ab) : compteur = 3 → ERREUR circuit breaker
        let result = breaker.check(&call_ab);
        assert!(
            result.is_err(),
            "les clés inversées doivent être détectées comme identiques"
        );
    }

    #[test]
    fn resets_on_different_call() {
        let mut breaker = CircuitBreaker::new();
        let same = call("bash", 99);
        let different = call("bash", 100);

        // 2 appels identiques : OK
        assert!(breaker.check(&same).is_ok());
        assert!(breaker.check(&same).is_ok());
        // Appel différent : reset → OK
        assert!(breaker.check(&different).is_ok());
        // Reprendre les appels identiques au début (compteur remis à 1)
        assert!(breaker.check(&same).is_ok());
        assert!(breaker.check(&same).is_ok());
        // Maintenant le 3ème identique consécutif => ERREUR
        assert!(breaker.check(&same).is_err());
    }
}
