import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { FORECAST_WORKBENCH_WINDOW } from "./forecast-workbench-constants";
import {
  isForecastWorkbenchSnapshot,
  type ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";

interface OpenForecastWorkbenchOptions {
  sessionId: string;
  analysisId: string | null;
  title: string;
}

export async function openForecastWorkbench({
  sessionId,
  analysisId,
  title,
}: OpenForecastWorkbenchOptions) {
  const snapshot = await invoke<ForecastWorkbenchSnapshot>("set_forecast_workbench_context", {
    sessionId,
    analysisId,
  });
  if (!isForecastWorkbenchSnapshot(snapshot)) {
    throw new Error("forecast-workbench-unavailable");
  }
  const windowTitle = snapshot.analysis_name
    ? `${title} — ${snapshot.analysis_name}`
    : title;
  const existing = await WebviewWindow.getByLabel(FORECAST_WORKBENCH_WINDOW.label);
  if (existing) {
    await existing.setTitle(windowTitle);
    await existing.show();
    await existing.setFocus();
    return;
  }
  const workbench = new WebviewWindow(FORECAST_WORKBENCH_WINDOW.label, {
    url: FORECAST_WORKBENCH_WINDOW.route,
    title: windowTitle,
    width: FORECAST_WORKBENCH_WINDOW.width,
    height: FORECAST_WORKBENCH_WINDOW.height,
    minWidth: FORECAST_WORKBENCH_WINDOW.minWidth,
    minHeight: FORECAST_WORKBENCH_WINDOW.minHeight,
    decorations: true,
    transparent: false,
    resizable: true,
  });
  await new Promise<void>((resolve, reject) => {
    const removeCreated = workbench.once("tauri://created", () => {
      cleanupTauriListener(removeError);
      resolve();
    });
    const removeError = workbench.once("tauri://error", () => {
      cleanupTauriListener(removeCreated);
      reject(new Error("forecast-workbench-unavailable"));
    });
  });
}
