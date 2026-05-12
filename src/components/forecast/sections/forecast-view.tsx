import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import { formatForecastValue, inferMetricMeta } from "../forecast-view-format";
import { ForecastChart } from "../charts/forecast-chart";
import { useForecastChartResize } from "../use-forecast-chart-resize";
import { KpiRow, PeriodCell, ValueCell } from "../forecast-view-widgets";
import "../forecast-view.css";

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
    series_ids?: string[];
    history: { date: string; value: number; series_id?: string | null }[];
  };
  predictions: { date: string; value: number; series_id?: string | null }[];
  quantiles: { q10: number[]; q50: number[]; q90: number[] };
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null } | null;
}

interface ForecastViewProps {
  analysisId: string;
  layers: ForecastLayerState;
}

export function ForecastView({ analysisId, layers }: ForecastViewProps) {
  const { t, i18n } = useTranslation();
  const [data, setData] = useState<ForecastResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedSeries, setSelectedSeries] = useState("");
  const chart = useForecastChartResize();

  useEffect(() => {
    invoke<ForecastResult>("get_forecast_analysis", { id: analysisId })
      .then(setData)
      .catch(() => setError(t("forecast.noAnalysis")));
  }, [analysisId, t]);

  if (error) return <div className="fc-error">{error}</div>;
  if (!data) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  const metric = inferMetricMeta(i18n.language, data.target_column, data.name);
  const activeSeries =
    selectedSeries && data.input_data.series_ids?.includes(selectedSeries)
      ? selectedSeries
      : data.input_data.series_ids?.[0] ?? "";
  const filtered = filterSeriesData(data, activeSeries);

  return (
    <div className="fc-view">
      {data.metrics && <KpiRow metrics={data.metrics} />}
      {data.input_data.series_ids && data.input_data.series_ids.length > 1 && (
        <div className="fc-view-toolbar">
          <label className="fc-view-toolbar-label" htmlFor="fc-view-series">
            {t("forecast.view.series")}
          </label>
          <select
            id="fc-view-series"
            className="fc-view-toolbar-select"
            value={activeSeries}
            onChange={(event) => setSelectedSeries(event.target.value)}
          >
            {data.input_data.series_ids.map((seriesId) => (
              <option key={seriesId} value={seriesId}>
                {seriesId}
              </option>
            ))}
          </select>
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
            }}
            layers={layers}
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

function filterSeriesData(data: ForecastResult, selectedSeries: string) {
  if (!data.input_data.series_ids || data.input_data.series_ids.length <= 1) {
    return {
      history: data.input_data.history,
      predictions: data.predictions,
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
    q10: indices.map((index) => data.quantiles.q10[index]).filter((value) => value !== undefined),
    q90: indices.map((index) => data.quantiles.q90[index]).filter((value) => value !== undefined),
  };
}
