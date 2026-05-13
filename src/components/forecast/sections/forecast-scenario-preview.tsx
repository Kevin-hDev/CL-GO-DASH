import { useTranslation } from "react-i18next";
import { ForecastChart } from "../charts/forecast-chart";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import type { ScenarioLine } from "../charts/forecast-chart-types";
import { buildScenarioVariableLines } from "./forecast-scenario-variable-lines";
import type {
  ForecastScenarioAnalysis,
  ForecastScenarioCovariateAdjustment,
} from "./forecast-scenario-types";
import "./forecast-scenario-preview.css";

interface ForecastScenarioPreviewProps {
  analysis: ForecastScenarioAnalysis;
  scenarioName: string;
  mode: "percent_adjustment" | "context_adjustment";
  adjustmentPercent: number;
  activeScenarioId: string | null;
  editingScenarioId: string | null;
  showDraftPreview: boolean;
  selectedSeries: string;
  targetSeriesId: string;
  contextAdjustments: ForecastScenarioCovariateAdjustment[];
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
  mode,
  adjustmentPercent,
  activeScenarioId,
  editingScenarioId,
  showDraftPreview,
  selectedSeries,
  targetSeriesId,
  contextAdjustments,
  chartHeight,
  isResizing,
}: ForecastScenarioPreviewProps) {
  const { t, i18n } = useTranslation();
  const factor = 1 + adjustmentPercent / 100;
  const seriesId = selectedSeries || analysis.input_data.series_ids?.[0] || "";
  const filtered = filterSeriesData(analysis, seriesId);
  const previewName = scenarioName.trim() || t("forecast.scenarios.preview");
  const savedScenarios =
    editingScenarioId && showDraftPreview
      ? []
      : buildSavedScenarios(analysis, activeScenarioId, editingScenarioId, seriesId);
  const previewScenario = showDraftPreview
    ? buildDraftScenario(
        analysis,
        filtered.predictions,
        previewName,
        factor,
        mode,
        editingScenarioId,
        seriesId,
      )
    : null;
  const chartScenarios = previewScenario ? [...savedScenarios, previewScenario] : savedScenarios;
  const variableLines = buildScenarioVariableLines({
    analysis,
    historyValues: filtered.history.map((point) => point.value),
    forecastValues: filtered.predictions.map((point) => point.value),
    selectedSeries: seriesId,
    activeScenarioId,
    draftAdjustments: contextAdjustments,
    draftTargetSeriesId: targetSeriesId || null,
    showDraftPreview,
  });
  const previewLayers = buildPreviewLayers(chartScenarios, variableLines);

  return (
    <div className="fcs-preview">
      <div
        className={`fcs-preview-chart ${isResizing ? "is-resizing" : ""}`}
        style={{ height: chartHeight, minHeight: chartHeight, maxHeight: chartHeight }}
      >
        <ForecastChart
          history={filtered.history}
          predictions={filtered.predictions}
          scenarios={chartScenarios}
          variables={variableLines}
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
            annotationUser: t("forecast.notes.userSource"),
            annotationLlm: t("forecast.notes.llmSource"),
          }}
          layers={previewLayers}
          mode="scenario"
        />
      </div>
    </div>
  );
}

function buildPreviewLayers(
  scenarios: ScenarioLine[],
  variables: ReturnType<typeof buildScenarioVariableLines>,
): ForecastLayerState {
  const layers: ForecastLayerState = { ...PREVIEW_LAYERS };
  for (const scenario of scenarios) {
    layers[`scenario-${scenario.id}`] = true;
  }
  for (const variable of variables) {
    layers[variable.id] = true;
  }
  return layers;
}

function buildSavedScenarios(
  analysis: ForecastScenarioAnalysis,
  activeScenarioId: string | null,
  editingScenarioId: string | null,
  selectedSeries: string,
): ScenarioLine[] {
  return analysis.scenarios
    .filter((scenario) => scenario.id === (editingScenarioId ?? activeScenarioId))
    .map((scenario) => ({
      id: scenario.id,
      name: scenario.name,
      predictions: selectedSeries
        ? scenario.predictions.filter((point) => point.series_id === selectedSeries)
        : scenario.predictions,
    }));
}

function buildDraftScenario(
  analysis: ForecastScenarioAnalysis,
  predictions: ForecastScenarioAnalysis["predictions"],
  previewName: string,
  factor: number,
  mode: "percent_adjustment" | "context_adjustment",
  editingScenarioId: string | null,
  selectedSeries: string,
): ScenarioLine | null {
  if (mode === "context_adjustment") {
    const saved = analysis.scenarios.find((scenario) => scenario.id === editingScenarioId);
    if (!saved) return null;
    return {
      id: saved.id,
      name: saved.name,
      predictions: selectedSeries
        ? saved.predictions.filter((point) => point.series_id === selectedSeries)
        : saved.predictions,
    };
  }
  return {
    id: "preview",
    name: previewName,
    predictions: predictions.map((point) => ({
      ...point,
      value: point.value * factor,
    })),
  };
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
