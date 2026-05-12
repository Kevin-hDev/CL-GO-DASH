import { useCallback, useEffect, useState, type Dispatch, type SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastLayerItem, ForecastLayerState } from "./forecast-layer-matrix";

interface ForecastLayerAnalysis {
  covariates_used?: string[];
  scenarios?: { id: string; name: string }[];
}

interface ForecastLayerSources {
  scenarioLayers: ForecastLayerItem[];
  covariateNames: string[];
}

const EMPTY_SOURCES: ForecastLayerSources = {
  scenarioLayers: [],
  covariateNames: [],
};

export function useForecastLayerSources(
  analysisId: string | null,
  setLayers: Dispatch<SetStateAction<ForecastLayerState>>
) {
  const [sources, setSources] = useState<ForecastLayerSources>(EMPTY_SOURCES);

  const refresh = useCallback(async () => {
    const nextSources = await loadSources(analysisId);
    applySources(nextSources, setSources, setLayers);
  }, [analysisId, setLayers]);

  useEffect(() => {
    let active = true;
    void loadSources(analysisId)
      .then((nextSources) => {
        if (active) applySources(nextSources, setSources, setLayers);
      })
      .catch(() => {
        if (active) applySources(EMPTY_SOURCES, setSources, setLayers);
      });
    return () => {
      active = false;
    };
  }, [analysisId, setLayers]);

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
    for (const name of sources.covariateNames) {
      const id = `variable-${name}`;
      if (next[id] === undefined) next[id] = false;
    }
    return next;
  });
}
