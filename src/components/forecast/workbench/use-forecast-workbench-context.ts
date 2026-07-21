import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type {
  ForecastWorkbenchContext,
  ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";
import { isForecastWorkbenchSnapshot } from "./forecast-workbench-types";

export function useForecastWorkbenchContext() {
  const [snapshot, setSnapshot] = useState<ForecastWorkbenchSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [failed, setFailed] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const next = await invoke<ForecastWorkbenchSnapshot | null>(
        "get_forecast_workbench_context",
      );
      if (next !== null && !isForecastWorkbenchSnapshot(next)) {
        throw new Error("invalid-forecast-workbench-context");
      }
      setSnapshot(next);
      setFailed(false);
    } catch {
      setFailed(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- backend snapshot hydration is intentional
    void refresh();
    const unlisten = listen<ForecastWorkbenchContext>(
      "forecast-workbench-context-changed",
      () => void refresh(),
    );
    return () => cleanupTauriListener(unlisten);
  }, [refresh]);

  return { snapshot, loading, failed, refresh };
}
