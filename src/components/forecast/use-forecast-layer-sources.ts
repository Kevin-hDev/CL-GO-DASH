import { useCallback, useEffect, useState, type Dispatch, type SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { ForecastLayerItem, ForecastLayerState } from "./forecast-layer-matrix";

interface ForecastLayerAnalysis {
  covariates_used?: string[];
  scenarios?: { id: string; name: string }[];
  ensemble?: object | null;
}

interface ForecastLayerSources {
  scenarioLayers: ForecastLayerItem[];
  comparisonLayers: ForecastLayerItem[];
  covariateNames: string[];
}

interface ForecastUpdatedEvent {
  analysis_id: string;
}

const EMPTY_SOURCES: ForecastLayerSources = {
  scenarioLayers: [],
  comparisonLayers: [],
  covariateNames: [],
};

export function useForecastLayerSources(
  analysisId: string | null,
  setLayers: Dispatch<SetStateAction<ForecastLayerState>>
) {
  const [sources, setSources] = useState<ForecastLayerSources>(EMPTY_SOURCES);

  const refresh = useCallback(async () => {
    try {
      const nextSources = await loadSources(analysisId);
      applySources(nextSources, setSources, setLayers);
    } catch {
      applySources(EMPTY_SOURCES, setSources, setLayers);
    }
  }, [analysisId, setLayers]);

  useEffect(() => {
    void refresh();
    const unlisten = listen<ForecastUpdatedEvent>("forecast-analysis-updated", (event) => {
      if (event.payload.analysis_id === analysisId) void refresh();
    });
    return () => cleanupTauriListener(unlisten);
  }, [analysisId, refresh]);

  return { sources, refresh };
}

async function loadSources(analysisId: string | null): Promise<ForecastLayerSources> {
  if (!analysisId) return EMPTY_SOURCES;
  const analysis = await invoke<ForecastLayerAnalysis>("get_forecast_analysis", { id: analysisId });
  const scenarioLayers: ForecastLayerItem[] = (analysis.scenarios ?? []).map((scenario) => ({
    id: `scenario-${scenario.id}`,
    label: scenario.name,
    interactive: true,
  }));
  return {
    scenarioLayers,
    comparisonLayers: analysis.ensemble ? [{
      id: "scenario-ensemble",
      label: i18n.t("forecast.view.ensembleSeries"),
      interactive: true,
    }] : [],
    covariateNames: analysis.covariates_used ?? [],
  };
}

function applySources(
  sources: ForecastLayerSources,
  setSources: Dispatch<SetStateAction<ForecastLayerSources>>,
  setLayers: Dispatch<SetStateAction<ForecastLayerState>>
) {
  setSources(sources);
  setLayers((current) => {
    const next = { ...current };
    for (const layer of sources.scenarioLayers) {
      if (next[layer.id] === undefined) next[layer.id] = true;
    }
    for (const layer of sources.comparisonLayers) {
      if (next[layer.id] === undefined) next[layer.id] = true;
    }
    for (const name of sources.covariateNames) {
      const id = `variable-${name}`;
      if (next[id] === undefined) next[id] = false;
    }
    return next;
  });
}
