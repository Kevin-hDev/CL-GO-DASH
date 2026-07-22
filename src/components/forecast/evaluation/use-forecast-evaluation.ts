import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { EvaluationAnalysis } from "./forecast-evaluation-types";

interface ForecastUpdatedEvent {
  analysis_id: string;
}

export function useForecastEvaluation(analysisId: string) {
  const [analysis, setAnalysis] = useState<EvaluationAnalysis | null>(null);
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [loadFailed, setLoadFailed] = useState(false);
  const [runFailed, setRunFailed] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const next = await invoke<EvaluationAnalysis>("get_forecast_analysis", { id: analysisId });
      setAnalysis(next);
      setLoadFailed(false);
    } catch {
      setLoadFailed(true);
    } finally {
      setLoading(false);
    }
  }, [analysisId]);

  const run = useCallback(async () => {
    setRunFailed(false);
    setRunning(true);
    try {
      const next = await invoke<EvaluationAnalysis>("run_forecast_backtest", {
        request: { analysis_id: analysisId, model_ids: [], max_windows: 3 },
      });
      setAnalysis(next);
      setRunFailed(false);
    } catch {
      setRunFailed(true);
    } finally {
      setRunning(false);
    }
  }, [analysisId]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- backend hydration is intentional
    void refresh();
    const unlisten = listen<ForecastUpdatedEvent>("forecast-analysis-updated", (event) => {
      if (event.payload.analysis_id === analysisId) void refresh();
    });
    return () => cleanupTauriListener(unlisten);
  }, [analysisId, refresh]);

  return { analysis, loading, running, loadFailed, runFailed, run };
}
