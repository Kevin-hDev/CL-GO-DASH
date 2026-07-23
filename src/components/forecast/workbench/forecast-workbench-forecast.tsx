import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  buildForecastLayerGroups,
  createInitialLayerState,
  type ForecastLayerState,
} from "../forecast-layer-matrix";
import { ForecastViewFilters } from "../forecast-view-filters";
import { ForecastView } from "../sections/forecast-view";
import {
  filterForecastSeriesData,
  type ForecastViewResult,
} from "../sections/forecast-view-data";
import { useForecastLayerSources } from "../use-forecast-layer-sources";
import { useForecastResult } from "../use-forecast-result";
import { ForecastChartCard } from "../charts/forecast-chart-card";
import { ForecastFanChart } from "../charts/forecast-fan-chart";
import { ForecastSeasonalityChart } from "../charts/forecast-seasonality-chart";
import { SEASONALITY_MIN_HISTORY } from "../charts/forecast-seasonality-data";
import "./forecast-workbench-forecast.css";

export function ForecastWorkbenchForecast({ analysisId }: { analysisId: string }) {
  const { t } = useTranslation();
  const [layers, setLayers] = useState<ForecastLayerState>(createInitialLayerState);
  const [fanResize, setFanResize] = useState(0);
  const [seasonalityResize, setSeasonalityResize] = useState(0);
  const { sources } = useForecastLayerSources(analysisId, setLayers);
  const groups = buildForecastLayerGroups(sources, t);
  const { data } = useForecastResult<ForecastViewResult>(
    analysisId,
    t("forecast.noAnalysis"),
  );
  const filtered = data
    ? filterForecastSeriesData(data, data.input_data.series_ids?.[0] ?? "", [])
    : null;

  return (
    <div className="fcwf-root">
      <div className="fcwf-toolbar">
        <ForecastViewFilters groups={groups} layers={layers} onChange={setLayers} />
      </div>
      <div className="fcwf-stack">
        <ForecastChartCard title={t("forecast.chartCard.main")}>
          <ForecastView analysisId={analysisId} layers={layers} />
        </ForecastChartCard>
        {filtered && filtered.predictions.length > 0 ? (
          <ForecastChartCard
            title={t("forecast.chartCard.fan")}
            onExpanded={() => setFanResize((value) => value + 1)}
          >
            <ForecastFanChart analysisId={analysisId} resizeSignal={fanResize} />
          </ForecastChartCard>
        ) : null}
        {filtered && filtered.history.length > SEASONALITY_MIN_HISTORY ? (
          <ForecastChartCard
            title={t("forecast.chartCard.seasonality")}
            onExpanded={() => setSeasonalityResize((value) => value + 1)}
          >
            <ForecastSeasonalityChart
              analysisId={analysisId}
              resizeSignal={seasonalityResize}
            />
          </ForecastChartCard>
        ) : null}
      </div>
    </div>
  );
}
