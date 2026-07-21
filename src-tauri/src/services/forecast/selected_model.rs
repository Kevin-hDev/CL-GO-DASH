use super::{
    selection_policy::{self, ForecastSelectionMode},
    types::ForecastRequest,
    validation,
};

pub fn apply_required(request: &mut ForecastRequest) -> Result<String, String> {
    let policy = selection_policy::get()?;
    apply_policy(request, policy)
}

fn apply_policy(
    request: &mut ForecastRequest,
    policy: selection_policy::ForecastSelectionPolicy,
) -> Result<String, String> {
    match policy.mode {
        ForecastSelectionMode::Manual => {
            let model = policy
                .manual_model_id
                .ok_or_else(|| "Aucun modèle Forecast sélectionné".to_string())?;
            if request
                .model
                .as_deref()
                .is_some_and(|requested| requested != model)
            {
                return Err("Le modèle demandé ne correspond pas à la sélection manuelle".into());
            }
            request.model = Some(model.clone());
            Ok(model)
        }
        ForecastSelectionMode::Auto => {
            let model = request
                .model
                .as_deref()
                .ok_or_else(|| "Aucun modèle compatible choisi pour Auto".to_string())?;
            validation::validate_runnable_model_id(model)?;
            let spec = super::catalog::find_model(model).ok_or("Modèle inconnu")?;
            if spec.is_cloud && !policy.allow_cloud_in_auto {
                return Err("Les modèles cloud ne sont pas autorisés en mode Auto".into());
            }
            Ok(model.to_string())
        }
    }
}

#[cfg(test)]
#[path = "selected_model_tests.rs"]
mod tests;
