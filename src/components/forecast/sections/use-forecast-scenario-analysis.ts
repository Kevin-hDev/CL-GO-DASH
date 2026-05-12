import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastScenarioAnalysis } from "./forecast-scenario-types";

interface UseForecastScenarioAnalysisArgs {
  analysisId: string;
  onLoaded: (analysis: ForecastScenarioAnalysis) => void;
  onFailed: () => void;
}

export function useForecastScenarioAnalysis({
  analysisId,
  onLoaded,
  onFailed,
}: UseForecastScenarioAnalysisArgs) {
  useEffect(() => {
    let active = true;
    void invoke<ForecastScenarioAnalysis>("get_forecast_analysis", { id: analysisId })
      .then((analysis) => {
        if (active) onLoaded(analysis);
      })
      .catch(() => {
        if (active) onFailed();
      });
    return () => {
      active = false;
    };
  }, [analysisId, onFailed, onLoaded]);
}
