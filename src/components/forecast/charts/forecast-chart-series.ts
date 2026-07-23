import type { ForecastLayerState } from "../forecast-layer-matrix";
import type {
  ForecastChartOptionArgs,
  ForecastChartPalette,
  ScenarioLine,
  TimelineEntry,
  VariableLine,
} from "./forecast-chart-types";
import {
  FORECAST_CHART_LINE_MONOTONE_AXIS,
  FORECAST_CHART_LINE_SMOOTHING,
} from "./forecast-chart-line-style";

export function buildSeries(
  timeline: TimelineEntry[],
  separatorTimestamp: number | null,
  palette: ForecastChartPalette,
  scenarios: ScenarioLine[],
  variables: VariableLine[],
  labels: ForecastChartOptionArgs["labels"],
  layers: ForecastLayerState,
  activeAnnotationId: string | null,
) {
  const lower = pointData(timeline, (entry) =>
    layers.confidence ? entry.lowerValue : null,
  );
  const band = pointData(timeline, (entry) =>
    layers.confidence && entry.lowerValue != null && entry.upperValue != null
      ? entry.upperValue - entry.lowerValue
      : null,
  );

  return [
    seriesLine(labels.confidence, lower, { opacity: 0 }, undefined, "confidence-90", false),
    { ...seriesLine(labels.confidence, band, { opacity: 0 }, palette.band90, "confidence-90", false), areaStyle: { color: palette.band90 } },
    seriesLine(
      labels.history,
      pointData(timeline, (entry) => (layers.history ? entry.historyValue : null)),
      { color: palette.lineHistory, width: 2 },
      undefined,
      undefined,
      false
    ),
    {
      ...seriesLine(
        labels.forecast,
        pointData(timeline, (entry) => (layers.forecast ? entry.forecastValue : null)),
        { color: palette.linePredict, width: 1.8 },
        undefined,
        undefined,
        true
      ),
      itemStyle: { color: palette.pointPredict, borderColor: palette.linePredict, borderWidth: 2 },
      markLine: separatorTimestamp != null && layers.forecast ? {
        symbol: ["none", "none"],
        silent: true,
        label: { show: false },
        lineStyle: { color: palette.separator, type: "dashed" as const, width: 1 },
        data: [{ xAxis: separatorTimestamp }],
      } : undefined,
    },
    ...scenarios.map((scenario, index) =>
      seriesLine(
        scenario.name,
        pointData(timeline, (entry) => (
          layers[`scenario-${scenario.id}`]
            ? entry.scenarioValues.find((value) => value.id === scenario.id)?.value ?? null
            : null
        )),
        { color: scenarioColor(palette, index), width: 1.5, type: "dashed" },
        undefined,
        undefined,
        false
      )
    ),
    ...variables.map((variable, index) =>
      seriesLine(
        variable.name,
        pointData(timeline, (entry) =>
          layers[variable.id]
            ? entry.variableValues.find((value) => value.id === variable.id)?.value ?? null
            : null,
        ),
        { color: variableColor(palette, index), width: 1.2, type: "dotted" },
        undefined,
        undefined,
        false
      )
    ),
    annotationSeries(
      "annotation-user",
      labels.annotationUser,
      timeline,
      palette.annotationUser,
      "user",
      activeAnnotationId,
    ),
    annotationSeries(
      "annotation-llm",
      labels.annotationLlm,
      timeline,
      palette.annotationLlm,
      "llm",
      activeAnnotationId,
    ),
  ];
}

function seriesLine(
  name: string,
  data: ChartPoint[],
  lineStyle: Record<string, unknown>,
  areaColor?: string,
  stack?: string,
  showSymbol?: boolean
) {
  return {
    name,
    type: "line" as const,
    data,
    stack,
    symbol: "circle" as const,
    showSymbol: Boolean(showSymbol),
    symbolSize: 5,
    smooth: FORECAST_CHART_LINE_SMOOTHING,
    smoothMonotone: FORECAST_CHART_LINE_MONOTONE_AXIS,
    connectNulls: false,
    lineStyle,
    areaStyle: areaColor ? { color: areaColor } : undefined,
    emphasis: { disabled: true },
  };
}

type ChartPoint = [number, number] | null;

function pointData(
  timeline: TimelineEntry[],
  valueForEntry: (entry: TimelineEntry) => number | null,
): ChartPoint[] {
  return timeline.map((entry) => {
    const value = valueForEntry(entry);
    return value == null ? null : [entry.timestamp, value];
  });
}

function scenarioColor(palette: ForecastChartPalette, index: number): string {
  return palette.scenarios[index % Math.max(palette.scenarios.length, 1)] || palette.lineHistory;
}

function variableColor(palette: ForecastChartPalette, index: number): string {
  return palette.variables[index % Math.max(palette.variables.length, 1)] || palette.inkMuted;
}

function annotationSeries(
  id: string,
  name: string,
  timeline: TimelineEntry[],
  color: string,
  source: "user" | "llm",
  activeAnnotationId: string | null,
) {
  return {
    id,
    name,
    type: "scatter" as const,
    data: timeline.map((entry) => {
      const annotations = entry.annotationValues.filter(
        (annotation) => annotation.source === source,
      );
      const value = markerValue(entry);
      if (!annotations.length || value == null) return null;
      return {
        value: [entry.timestamp, value],
        annotationIds: annotations.map((annotation) => annotation.id),
        symbolSize: annotations.some((annotation) => annotation.id === activeAnnotationId) ? 11 : 8,
      };
    }),
    symbol: "circle" as const,
    symbolSize: 8,
    itemStyle: { color, borderColor: color, borderWidth: 1 },
    emphasis: { scale: 1.25 },
    z: 8,
  };
}

function markerValue(entry: TimelineEntry): number | null {
  if (entry.historyValue != null) return entry.historyValue;
  if (entry.forecastValue != null) return entry.forecastValue;
  if (entry.lowerValue != null && entry.upperValue != null) {
    return (entry.lowerValue + entry.upperValue) / 2;
  }
  return entry.scenarioValues[0]?.value ?? entry.variableValues[0]?.value ?? null;
}
