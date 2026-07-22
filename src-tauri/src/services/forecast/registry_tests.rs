use super::{catalog, registry};

#[test]
fn every_catalog_model_has_runtime_capabilities() {
    for model in catalog::FORECAST_MODELS {
        let runtime = registry::find_runtime(model.id)
            .unwrap_or_else(|| panic!("missing runtime for {}", model.id));
        assert_eq!(runtime.family_id, model.family_id);
        assert!(
            !runtime.config_params.is_empty(),
            "{} has no config params",
            model.id
        );
    }
}

#[test]
fn runtime_capabilities_match_key_adapters() {
    let chronos_bolt = registry::find_runtime("chronos-bolt-tiny").unwrap();
    assert!(!chronos_bolt.capabilities.past_covariates);
    assert!(!chronos_bolt.capabilities.multivariate);

    let chronos_2 = registry::find_runtime("chronos-2").unwrap();
    assert!(chronos_2.capabilities.past_covariates);
    assert!(chronos_2.capabilities.future_covariates);
    assert!(chronos_2.capabilities.multivariate);

    let toto = registry::find_runtime("toto-2.0-2.5b").unwrap();
    assert!(toto.capabilities.past_covariates);
    assert!(toto.capabilities.future_covariates);
    assert!(toto.config_params.contains(&"decode_block_size"));

    let timesfm = registry::find_runtime("timesfm-2.5-200m").unwrap();
    assert!(!timesfm.capabilities.past_covariates);
    assert!(!timesfm.capabilities.multivariate);
}

#[test]
fn every_prediction_adapter_supports_rolling_backtests() {
    for runtime in registry::FORECAST_RUNTIMES {
        if registry::has_predict_adapter(runtime) {
            assert!(
                runtime.capabilities.backtesting_ready,
                "{}",
                runtime.model_id
            );
        }
    }
}
