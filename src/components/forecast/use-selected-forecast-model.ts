import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useSelectedForecastModel() {
  const [selectedModelId, setSelectedModelId] = useState("");
  const [ready, setReady] = useState(false);

  const selectModel = useCallback((modelId: string) => {
    setSelectedModelId(modelId);
    void invoke("set_selected_forecast_model", { name: modelId }).catch(() => undefined);
  }, []);

  useEffect(() => {
    let mounted = true;
    void invoke<string | null>("get_selected_forecast_model")
      .then((modelId) => {
        if (mounted && modelId) setSelectedModelId(modelId);
      })
      .catch(() => undefined)
      .finally(() => {
        if (mounted) setReady(true);
      });
    const unlisten = listen<string>("forecast-selected-model-changed", (event) => {
      setSelectedModelId(event.payload);
    });
    return () => {
      mounted = false;
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  return { selectedModelId, selectModel, ready };
}
