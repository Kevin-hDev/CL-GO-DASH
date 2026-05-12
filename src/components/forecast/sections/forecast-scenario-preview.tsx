import { useTranslation } from "react-i18next";
import { ForecastChart } from "../charts/forecast-chart";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import type { ScenarioLine } from "../charts/forecast-chart-types";
import type { ForecastScenarioAnalysis } from "./forecast-scenario-types";
import "./forecast-scenario-preview.css";

interface ForecastScenarioPreviewProps {
  analysis: ForecastScenarioAnalysis;
  scenarioName: string;
  adjustmentPercent: number;
  selectedSeries: string;
  chartHeight: number;
  isResizing: boolean;
}

const PREVIEW_LAYERS: ForecastLayerState = {
  history: true,
  forecast: true,
  confidence: true,
  "scenario-preview": true,
};

export function ForecastScenarioPreview({
  analysis,
  scenarioName,
  adjustmentPercent,
  selectedSeries,
  chartHeight,
  isResizing,
}: ForecastScenarioPreviewProps) {
  const { t, i18n } = useTranslation();
  const factor = 1 + adjustmentPercent / 100;
  const seriesId = selectedSeries || analysis.input_data.series_ids?.[0] || "";
  const filtered = filterSeriesData(analysis, seriesId);
  const previewName = scenarioName.trim() || t("forecast.scenarios.preview");
  const previewScenario: ScenarioLine = {
    id: "preview",
    name: previewName,
    predictions: filtered.predictions.map((point) => ({
      ...point,
      value: point.value * factor,
    })),
  };

  return (
    <div className="fcs-preview">
      <div
        className={`fcs-preview-chart ${isResizing ? "is-resizing" : ""}`}
        style={{ height: chartHeight, minHeight: chartHeight, maxHeight: chartHeight }}
      >
        <ForecastChart
          history={filtered.history}
          predictions={filtered.predictions}
          scenarios={[previewScenario]}
          quantiles={filtered.quantiles}
          frequency={analysis.frequency}
          endDate={analysis.input_summary.end}
          locale={i18n.language}
          targetColumn={analysis.target_column}
          fallbackName={previewName}
          labels={{
            history: t("forecast.view.historySeries"),
            forecast: t("forecast.view.forecastSeries"),
            confidence: t("forecast.view.confidenceRange"),
          }}
          layers={PREVIEW_LAYERS}
        />
      </div>
    </div>
  );
}

function filterSeriesData(analysis: ForecastScenarioAnalysis, selectedSeries: string) {
  if (!analysis.input_data.series_ids || analysis.input_data.series_ids.length <= 1) {
    return {
      history: analysis.input_data.history,
      predictions: analysis.predictions,
      quantiles: analysis.quantiles,
    };
  }

  const indices: number[] = [];
  const predictions = analysis.predictions.filter((point, index) => {
    const match = point.series_id === selectedSeries;
    if (match) indices.push(index);
    return match;
  });

  return {
    history: analysis.input_data.history.filter((point) => point.series_id === selectedSeries),
    predictions,
    quantiles: {
      q10: indices.map((index) => analysis.quantiles.q10[index]).filter((value) => value !== undefined),
      q90: indices.map((index) => analysis.quantiles.q90[index]).filter((value) => value !== undefined),
    },
  };
}
