import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import { formatForecastValue, inferMetricMeta } from "../forecast-view-format";
import { ForecastChart } from "../charts/forecast-chart";
import { useForecastChartResize } from "../use-forecast-chart-resize";
import { KpiRow, PeriodCell, ValueCell } from "../forecast-view-widgets";
import { buildForecastVariableLines } from "../forecast-variable-lines";
import { useForecastResult } from "../use-forecast-result";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";
import "../forecast-view.css";
import "../forecast-view-table.css";

interface ForecastResult {
  id: string;
  name: string;
  target_column?: string;
  series_column?: string | null;
  model: string;
  horizon: number;
  frequency: string;
  input_summary: {
    end: string;
  };
  input_data: {
    rows?: Record<string, unknown>[];
    series_ids?: string[];
    history: { date: string; value: number; series_id?: string | null }[];
  };
  covariates_used?: string[];
  predictions: { date: string; value: number; series_id?: string | null }[];
  quantiles: { q10: number[]; q50: number[]; q90: number[] };
  scenarios: {
    id: string;
    name: string;
    predictions: { date: string; value: number; series_id?: string | null }[];
  }[];
  ensemble?: {
    predictions: { date: string; value: number; series_id?: string | null }[];
  } | null;
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null } | null;
}

interface ForecastViewProps {
  analysisId: string;
  layers: ForecastLayerState;
}

export function ForecastView({ analysisId, layers }: ForecastViewProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastResult>(analysisId, t("forecast.noAnalysis"));
  const [selectedSeries, setSelectedSeries] = useState("");
  const chart = useForecastChartResize();

  if (error) return <div className="fc-error">{error}</div>;
  if (!data) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  const metric = inferMetricMeta(i18n.language, data.target_column, data.name);
  const activeSeries =
    selectedSeries && data.input_data.series_ids?.includes(selectedSeries)
      ? selectedSeries
      : data.input_data.series_ids?.[0] ?? "";
  const scenarioLines = [
    ...(data.scenarios ?? []),
    ...(data.ensemble ? [{
      id: "ensemble",
      name: t("forecast.view.ensembleSeries"),
      predictions: data.ensemble.predictions,
    }] : []),
  ];
  const filtered = filterSeriesData(data, activeSeries, scenarioLines);
  const variables = buildForecastVariableLines({
    rows: data.input_data.rows ?? [],
    covariates: data.covariates_used ?? [],
    targetColumn: data.target_column ?? "",
    seriesColumn: data.series_column,
    historyValues: filtered.history.map((point) => point.value),
    forecastValues: filtered.predictions.map((point) => point.value),
    selectedSeries: activeSeries,
  });

  return (
    <div className="fc-view">
      {data.metrics && <KpiRow metrics={data.metrics} />}
      {data.input_data.series_ids && data.input_data.series_ids.length > 1 && (
        <div className="fc-view-toolbar">
          <span className="fc-view-toolbar-label">
            {t("forecast.view.series")}
          </span>
          <ForecastScenarioMenuSelect
            className="fc-view-toolbar-menu"
            value={activeSeries}
            options={data.input_data.series_ids.map((seriesId) => ({
              value: seriesId,
              label: seriesId,
            }))}
            onChange={setSelectedSeries}
          />
        </div>
      )}
      <div
        className={`fc-chart-area ${chart.isResizing ? "is-resizing" : ""}`}
        style={{ height: chart.chartHeight, minHeight: chart.chartHeight, maxHeight: chart.chartHeight }}
      >
        <div className="fc-chart-placeholder">
          <ForecastChart
            history={filtered.history}
            predictions={filtered.predictions}
            scenarios={filtered.scenarios}
            variables={variables}
            quantiles={{ q10: filtered.q10, q90: filtered.q90 }}
            frequency={data.frequency}
            endDate={data.input_summary.end}
            locale={i18n.language}
            targetColumn={data.target_column}
            fallbackName={data.name}
            labels={{
              history: t("forecast.view.historySeries"),
              forecast: t("forecast.view.forecastSeries"),
              confidence: t("forecast.view.confidenceRange"),
              annotationUser: t("forecast.notes.userSource"),
              annotationLlm: t("forecast.notes.llmSource"),
            }}
            layers={layers}
            mode="main"
          />
        </div>
      </div>
      <div
        className="fc-chart-resize"
        onPointerDown={chart.startResize}
        onDoubleClick={chart.resetHeight}
      />
      <div className="fc-predictions-table">
        <div className="fc-table-head">
          <span>{t("forecast.view.period")}</span>
          <span>{metric.columnTitle}</span>
        </div>
        <div className="fc-table-body">
          {filtered.predictions.map((p, i) => (
            <div key={i} className="fc-table-row">
              <PeriodCell
                index={i}
                rawDate={p.date}
                endDate={data.input_summary.end}
                frequency={data.frequency}
                locale={i18n.language}
              />
              <ValueCell
                unitLabel={metric.unitLabel}
                formattedValue={formatForecastValue(p.value, i18n.language, metric)}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function filterSeriesData(
  data: ForecastResult,
  selectedSeries: string,
  scenarios: ForecastResult["scenarios"],
) {
  if (!data.input_data.series_ids || data.input_data.series_ids.length <= 1) {
    return {
      history: data.input_data.history,
      predictions: data.predictions,
      scenarios,
      q10: data.quantiles.q10,
      q90: data.quantiles.q90,
    };
  }

  const seriesId = selectedSeries || data.input_data.series_ids[0];
  const indices: number[] = [];
  const predictions = data.predictions.filter((point, index) => {
    const match = point.series_id === seriesId;
    if (match) indices.push(index);
    return match;
  });

  return {
    history: data.input_data.history.filter((point) => point.series_id === seriesId),
    predictions,
    scenarios: scenarios.map((scenario) => ({
      ...scenario,
      predictions: scenario.predictions.filter((point) => point.series_id === seriesId),
    })),
    q10: indices.map((index) => data.quantiles.q10[index]).filter((value) => value !== undefined),
    q90: indices.map((index) => data.quantiles.q90[index]).filter((value) => value !== undefined),
  };
}
