use crate::services::forecast::evaluation::{self, BacktestRequest};
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::types::ForecastResult;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn run_forecast_backtest(
    app: AppHandle,
    request: BacktestRequest,
    chronos: State<'_, ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let analysis = evaluation::run(request, chronos.inner()).await?;
    let _ = app.emit(
        "forecast-analysis-updated",
        serde_json::json!({ "analysis_id": analysis.id, "session_id": analysis.session_id }),
    );
    Ok(analysis)
}
