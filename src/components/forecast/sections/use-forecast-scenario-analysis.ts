import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { ForecastScenarioAnalysis } from "./forecast-scenario-types";

interface ForecastUpdatedEvent {
  analysis_id: string;
}

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
    const refresh = () => {
      void invoke<ForecastScenarioAnalysis>("get_forecast_analysis", { id: analysisId })
        .then((analysis) => {
          if (active) onLoaded(analysis);
        })
        .catch(() => {
          if (active) onFailed();
        });
    };
    refresh();
    const unlisten = listen<ForecastUpdatedEvent>("forecast-analysis-updated", (event) => {
      if (event.payload.analysis_id === analysisId) refresh();
    });
    return () => {
      active = false;
      cleanupTauriListener(unlisten);
    };
  }, [analysisId, onFailed, onLoaded]);
}
