use crate::services::forecast::workbench_context::{
    self, ForecastWorkbenchContext, ForecastWorkbenchSnapshot,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn set_forecast_workbench_context(
    app: AppHandle,
    session_id: String,
    analysis_id: Option<String>,
) -> Result<ForecastWorkbenchSnapshot, String> {
    let snapshot = workbench_context::set(session_id, analysis_id).await?;
    emit_context(&app, &snapshot.context)?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn get_forecast_workbench_context() -> Result<Option<ForecastWorkbenchSnapshot>, String> {
    workbench_context::get().await
}

fn emit_context(app: &AppHandle, context: &ForecastWorkbenchContext) -> Result<(), String> {
    app.emit("forecast-workbench-context-changed", context)
        .map_err(|_| "Impossible d'actualiser Forecast".to_string())
}
