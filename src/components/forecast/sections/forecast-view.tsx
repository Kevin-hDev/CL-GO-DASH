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
import {
  buildForecastLayerAnnotations,
  filterForecastSeriesData,
  type ForecastViewResult,
} from "./forecast-view-data";
import "../forecast-view.css";
import "../forecast-view-table.css";

interface ForecastViewProps {
  analysisId: string;
  layers: ForecastLayerState;
}

export function ForecastView({ analysisId, layers }: ForecastViewProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastViewResult>(analysisId, t("forecast.noAnalysis"));
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
  const filtered = filterForecastSeriesData(data, activeSeries, scenarioLines);
  const annotations = buildForecastLayerAnnotations(data, activeSeries, {
    anomaly: t("forecast.view.filters.residualAnomalies"),
    quality: t("forecast.view.filters.dataQualityIssues"),
  });
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
            annotations={annotations}
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
              today: t("forecast.chart.today"),
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
