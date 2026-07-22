import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

interface ForecastUpdatedEvent {
  analysis_id: string;
}

export function useForecastResult<T>(analysisId: string, errorMessage: string) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    try {
      setData(await invoke<T>("get_forecast_analysis", { id: analysisId }));
      setError(null);
    } catch {
      setError(errorMessage);
    }
  }, [analysisId, errorMessage]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- backend hydration is intentional
    void refresh();
    const unlisten = listen<ForecastUpdatedEvent>("forecast-analysis-updated", (event) => {
      if (event.payload.analysis_id === analysisId) void refresh();
    });
    return () => cleanupTauriListener(unlisten);
  }, [analysisId, refresh]);

  return { data, error };
}
