import type { EChartsOption } from "echarts";
import { buildPeriodMeta, inferMetricMeta, type ForecastMetricMeta } from "../forecast-view-format";
import { formatAxisValue, formatTooltip } from "./forecast-chart-format";
import { buildSeries } from "./forecast-chart-series";
import type { ForecastChartOptionArgs, TimelineEntry } from "./forecast-chart-types";

export function buildForecastChartOption(args: ForecastChartOptionArgs): EChartsOption {
  const metric = inferMetricMeta(args.locale, args.targetColumn, args.fallbackName);
  const timeline = buildTimeline(args);
  const separatorIndex = args.history.length > 0 ? args.history.length - 1 : null;
  const yAxisBounds = buildYAxisBounds(timeline, args.layers);

  return {
    animationDuration: 250,
    animationDurationUpdate: 250,
    grid: { left: 18, right: 18, top: 18, bottom: 38, containLabel: true },
    tooltip: {
      trigger: "axis",
      backgroundColor: "rgba(18, 18, 18, 0.94)",
      borderColor: args.palette.edge,
      borderWidth: 1,
      textStyle: { color: "#f5f5f5", fontFamily: "var(--font-sans)" },
      formatter: (raw) => formatTooltip(raw, timeline, metric, args.labels, args.locale),
    },
    xAxis: buildXAxis(timeline, args.palette.edge, args.palette.inkMuted),
    yAxis: buildYAxis(
      args.locale,
      metric,
      args.palette.edge,
      args.palette.inkMuted,
      yAxisBounds,
    ),
    dataZoom: [
      {
        type: "inside",
        xAxisIndex: 0,
        start: args.zoomWindow.start,
        end: args.zoomWindow.end,
        filterMode: "none",
        zoomOnMouseWheel: true,
        moveOnMouseWheel: false,
        moveOnMouseMove: false,
      },
    ],
    series: buildSeries(
      timeline,
      separatorIndex,
      args.palette,
      args.scenarios,
      args.variables,
      args.labels.confidence,
      args.layers
    ),
  };
}

function buildXAxis(timeline: TimelineEntry[], edge: string, inkMuted: string) {
  return {
    type: "category" as const,
    data: timeline.map((entry) => entry.axisLabel),
    boundaryGap: false,
    axisLine: { lineStyle: { color: edge } },
    axisTick: { show: false },
    axisLabel: { color: inkMuted, fontSize: 11, margin: 10 },
  };
}

function buildYAxis(
  locale: string,
  metric: ForecastMetricMeta,
  edge: string,
  inkMuted: string,
  bounds: { min: number; max: number } | null,
) {
  return {
    type: "value" as const,
    scale: true,
    min: bounds?.min,
    max: bounds?.max,
    splitLine: { lineStyle: { color: edge, opacity: 0.5 } },
    axisLine: { show: false },
    axisTick: { show: false },
    axisLabel: {
      color: inkMuted,
      fontSize: 11,
      formatter: (value: number) => formatAxisValue(value, locale, metric),
    },
  };
}

function buildYAxisBounds(
  timeline: TimelineEntry[],
  layers: ForecastChartOptionArgs["layers"],
) {
  const values: number[] = [];
  for (const entry of timeline) {
    if (entry.historyValue != null && layers.history) values.push(entry.historyValue);
    if (entry.forecastValue != null && layers.forecast) values.push(entry.forecastValue);
    if (layers.confidence) {
      if (entry.lowerValue != null) values.push(entry.lowerValue);
      if (entry.upperValue != null) values.push(entry.upperValue);
    }
    for (const scenario of entry.scenarioValues) values.push(scenario.value);
    for (const variable of entry.variableValues) values.push(variable.value);
  }
  if (!values.length) return null;
  let min = values[0];
  let max = values[0];
  for (const value of values) {
    if (value < min) min = value;
    if (value > max) max = value;
  }
  const span = max - min;
  const padding = span <= 0 ? Math.max(Math.abs(max) * 0.08, 1) : span * 0.12;
  return {
    min: min - padding,
    max: max + padding,
  };
}

function buildTimeline(args: ForecastChartOptionArgs): TimelineEntry[] {
  return [
    ...args.history.map((point, historyIndex) => ({
      axisLabel: shortDate(point.date, args.locale),
      fullLabel: point.date,
      historyValue: point.value,
      forecastValue: null,
      scenarioValues: [],
      variableValues: args.variables
        .map((variable) => ({
          id: variable.id,
          name: variable.name,
          value: variable.values[historyIndex],
          rawValue: variable.rawValues[historyIndex],
        }))
        .filter(
          (variable): variable is { id: string; name: string; value: number; rawValue: number } =>
            typeof variable.value === "number" && typeof variable.rawValue === "number",
        ),
      lowerValue: null,
      upperValue: null,
    })),
    ...args.predictions.map((point, index) => {
      const period = buildPeriodMeta(index, point.date, args.endDate, args.frequency, args.locale);
      return {
        axisLabel: shortDate(point.date, args.locale),
        fullLabel: `${period.stepLabel} - ${period.secondaryLabel}`,
        historyValue: null,
        forecastValue: point.value,
        scenarioValues: args.scenarios
          .map((scenario) => ({
            id: scenario.id,
            name: scenario.name,
            value: scenario.predictions[index]?.value,
          }))
          .filter(
            (scenario): scenario is { id: string; name: string; value: number } =>
              args.layers[`scenario-${scenario.id}`] === true &&
              typeof scenario.value === "number",
          ),
        variableValues: args.variables
          .map((variable) => ({
            id: variable.id,
            name: variable.name,
            value: variable.values[args.history.length + index],
            rawValue: variable.rawValues[args.history.length + index],
          }))
          .filter(
            (variable): variable is { id: string; name: string; value: number; rawValue: number } =>
              args.layers[variable.id] === true &&
              typeof variable.value === "number" && typeof variable.rawValue === "number",
          ),
        lowerValue: args.quantiles.q10[index] ?? null,
        upperValue: args.quantiles.q90[index] ?? null,
      };
    }),
  ];
}

function shortDate(value: string, locale: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat(locale, { month: "2-digit", day: "2-digit" }).format(parsed);
}
