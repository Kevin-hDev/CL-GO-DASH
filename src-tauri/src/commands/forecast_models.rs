use crate::services::forecast::types::ModelDownloadProgress;
use crate::services::forecast::{catalog, model_manager, registry, sidecar_runtime, validation};
use crate::services::paths::data_dir;
use serde_json::Value;
use tauri::ipc::Channel;

#[tauri::command]
pub fn list_forecast_models() -> Value {
    let installed = model_manager::installed_models();
    let configured = crate::services::api_keys::list_configured();
    let providers: Vec<Value> = catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| {
            let mut value = serde_json::to_value(p).unwrap_or_default();
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    "configured".into(),
                    Value::Bool(configured.iter().any(|id| id == p.id)),
                );
            }
            value
        })
        .collect();
    let models: Vec<Value> = catalog::FORECAST_MODELS
        .iter()
        .map(|m| {
            let mut value = serde_json::to_value(m).unwrap_or_default();
            if let Some(object) = value.as_object_mut() {
                enrich_model_object(object, m, &installed, &configured);
            }
            value
        })
        .collect();
    serde_json::json!({
        "providers": providers,
        "models": models,
        "configured_provider_ids": configured
    })
}

fn enrich_model_object(
    object: &mut serde_json::Map<String, Value>,
    model: &catalog::ForecastModelSpec,
    installed: &[String],
    configured: &[String],
) {
    let runtime = registry::find_runtime(model.id);
    let installed_model = installed.contains(&model.id.to_string());
    let provider_configured = configured.iter().any(|id| id == model.provider_id);
    if let Some(runtime) = runtime {
        object.insert(
            "family_id".into(),
            Value::String(runtime.family_id.to_string()),
        );
        object.insert(
            "engine_kind".into(),
            serde_json::to_value(runtime.engine_kind).unwrap_or_default(),
        );
        object.insert(
            "capabilities".into(),
            serde_json::to_value(runtime.capabilities).unwrap_or_default(),
        );
    } else {
        object.insert(
            "capabilities".into(),
            serde_json::json!({
                "past_covariates": model.covariates,
                "future_covariates": model.covariates,
                "multivariate": model.multivariate,
                "probabilistic": true,
                "backtesting_ready": false,
                "anomalies_ready": false,
                "fine_tuning_ready": false,
            }),
        );
    }
    object.insert(
        "family_id".into(),
        Value::String(model.family_id.to_string()),
    );
    object.insert("installed".into(), Value::Bool(installed_model));
    object.insert(
        "installable".into(),
        Value::Bool(!model.is_cloud && (model.hf_repo.is_some() || model.github_repo.is_some())),
    );
    object.insert(
        "runnable".into(),
        Value::Bool(
            runtime.is_some()
                && runnable_state(model, runtime, installed_model, provider_configured),
        ),
    );
    object.insert(
        "runtime_ready".into(),
        Value::Bool(runtime_ready_state(
            runtime,
            installed_model,
            provider_configured,
        )),
    );
    object.insert(
        "size_on_disk".into(),
        Value::Number(model_manager::get_model_size(model.id).into()),
    );
    object.insert(
        "provider_configured".into(),
        Value::Bool(provider_configured),
    );
}

fn runnable_state(
    model: &catalog::ForecastModelSpec,
    runtime: Option<&registry::ForecastRuntimeSpec>,
    installed: bool,
    provider_configured: bool,
) -> bool {
    match runtime {
        Some(spec) if registry::is_cloud(spec) => {
            registry::has_predict_adapter(spec) && provider_configured
        }
        Some(spec) => registry::has_predict_adapter(spec) && !model.is_cloud && installed,
        None => false,
    }
}

fn runtime_ready_state(
    runtime: Option<&registry::ForecastRuntimeSpec>,
    installed: bool,
    provider_configured: bool,
) -> bool {
    let Some(spec) = runtime else {
        return false;
    };
    if registry::is_cloud(spec) {
        return registry::has_predict_adapter(spec) && provider_configured;
    }
    if !registry::has_predict_adapter(spec) {
        return false;
    }
    installed
        && sidecar_runtime::family_runtime_ready(
            &data_dir().join("forecast-sidecar"),
            spec.family_id,
        )
}

#[tauri::command]
pub async fn install_forecast_model(
    name: String,
    on_progress: Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    model_manager::install(&name, &on_progress).await
}

#[tauri::command]
pub async fn uninstall_forecast_model(name: String) -> Result<(), String> {
    validation::validate_model_id(&name)?;
    model_manager::uninstall(&name).await
}

#[tauri::command]
pub fn list_forecast_providers_catalog() -> Vec<Value> {
    catalog::FORECAST_PROVIDERS
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect()
}
