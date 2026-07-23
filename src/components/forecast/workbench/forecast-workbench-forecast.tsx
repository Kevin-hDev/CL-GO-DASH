import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  buildForecastLayerGroups,
  createInitialLayerState,
  type ForecastLayerState,
} from "../forecast-layer-matrix";
import { ForecastViewFilters } from "../forecast-view-filters";
import { ForecastView } from "../sections/forecast-view";
import { useForecastLayerSources } from "../use-forecast-layer-sources";
import "./forecast-workbench-forecast.css";

export function ForecastWorkbenchForecast({ analysisId }: { analysisId: string }) {
  const { t } = useTranslation();
  const [layers, setLayers] = useState<ForecastLayerState>(createInitialLayerState);
  const { sources } = useForecastLayerSources(analysisId, setLayers);
  const groups = buildForecastLayerGroups(sources, t);

  return (
    <div className="fcwf-root">
      <div className="fcwf-toolbar">
        <ForecastViewFilters groups={groups} layers={layers} onChange={setLayers} />
      </div>
      <div className="fcwf-view">
        <ForecastView analysisId={analysisId} layers={layers} />
      </div>
    </div>
  );
}
