import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  DEFAULT_FORECAST_SELECTION_POLICY,
  isForecastSelectionPolicy,
  type ForecastSelectionMode,
  type ForecastSelectionPolicy,
} from "./forecast-selection-types";

export function useForecastSelectionPolicy() {
  const [policy, setPolicy] = useState<ForecastSelectionPolicy>(
    DEFAULT_FORECAST_SELECTION_POLICY,
  );
  const [ready, setReady] = useState(false);

  const selectModel = useCallback((modelId: string) => {
    void invoke<ForecastSelectionPolicy>("set_selected_forecast_model", { name: modelId })
      .then((next) => {
        if (!isForecastSelectionPolicy(next)) throw new Error("invalid-forecast-policy");
        setPolicy(next);
      })
      .catch(() => showToast(i18n.t("forecast.selection.saveFailed")));
  }, []);

  const setMode = useCallback((mode: ForecastSelectionMode) => {
    void invoke<ForecastSelectionPolicy>("set_forecast_selection_mode", { mode })
      .then((next) => {
        if (!isForecastSelectionPolicy(next)) throw new Error("invalid-forecast-policy");
        setPolicy(next);
      })
      .catch(() => showToast(i18n.t("forecast.selection.saveFailed")));
  }, []);

  useEffect(() => {
    let mounted = true;
    void invoke<ForecastSelectionPolicy>("get_forecast_selection_policy")
      .then((next) => {
        if (!isForecastSelectionPolicy(next)) throw new Error("invalid-forecast-policy");
        if (mounted) setPolicy(next);
      })
      .catch(() => showToast(i18n.t("forecast.selection.loadFailed")))
      .finally(() => {
        if (mounted) setReady(true);
      });
    const unlisten = listen<ForecastSelectionPolicy>(
      "forecast-selection-policy-changed",
      (event) => {
        if (isForecastSelectionPolicy(event.payload)) setPolicy(event.payload);
      },
    );
    return () => {
      mounted = false;
      cleanupTauriListener(unlisten);
    };
  }, []);

  return {
    policy,
    selectedModelId: policy.manual_model_id ?? "",
    selectModel,
    setMode,
    ready,
  };
}
