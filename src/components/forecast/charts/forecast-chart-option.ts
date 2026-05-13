import type { EChartsOption } from "echarts";
import { inferMetricMeta, type ForecastMetricMeta } from "../forecast-view-format";
import { formatAxisValue, formatTooltip } from "./forecast-chart-format";
import { buildSeries } from "./forecast-chart-series";
import { buildTimeline } from "./forecast-chart-timeline";
import type { ForecastChartOptionArgs, TimelineEntry } from "./forecast-chart-types";

export function buildForecastChartOption(args: ForecastChartOptionArgs): EChartsOption {
  const metric = inferMetricMeta(args.locale, args.targetColumn, args.fallbackName);
  const timeline = buildTimeline(args);
  const separatorTimestamp = args.history.length > 0
    ? timeline[args.history.length - 1]?.timestamp ?? null
    : null;
  const yAxisBounds = buildYAxisBounds(timeline, args.layers);

  return {
    animation: false,
    grid: {
      left: args.compact ? 8 : 18,
      right: args.compact ? 8 : 18,
      top: args.compact ? 10 : 18,
      bottom: args.compact ? 18 : 38,
      containLabel: true,
    },
    tooltip: {
      trigger: "axis",
      backgroundColor: "rgba(18, 18, 18, 0.94)",
      borderColor: args.palette.edge,
      borderWidth: 1,
      textStyle: { color: "#f5f5f5", fontFamily: "var(--font-sans)" },
      formatter: (raw) => formatTooltip(raw, timeline, metric, args.labels, args.locale),
      confine: true,
      position: tooltipPosition,
    },
    xAxis: buildXAxis(args.locale, args.palette.edge, args.palette.inkMuted),
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
        realtime: true,
        throttle: 16,
        filterMode: "none",
        zoomOnMouseWheel: true,
        moveOnMouseWheel: false,
        moveOnMouseMove: true,
      },
    ],
    series: buildSeries(
      timeline,
      separatorTimestamp,
      args.palette,
      args.scenarios,
      args.variables,
      args.labels,
      args.layers,
      args.activeAnnotationId ?? null,
    ),
  };
}

function buildXAxis(locale: string, edge: string, inkMuted: string) {
  return {
    type: "time" as const,
    boundaryGap: ["0%", "0%"] as [string, string],
    axisLine: { lineStyle: { color: edge } },
    axisTick: { show: false },
    axisLabel: {
      color: inkMuted,
      fontSize: 11,
      margin: 10,
      formatter: (value: number) => formatAxisDate(value, locale),
    },
  };
}

function formatAxisDate(value: number, locale: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "";
  return new Intl.DateTimeFormat(locale, { month: "2-digit", day: "2-digit" }).format(parsed);
}

function tooltipPosition(
  point: number[],
  _params: unknown,
  _dom: unknown,
  _rect: unknown,
  size: { contentSize: number[]; viewSize: number[] },
): [number, number] {
  const gap = 14;
  const margin = 8;
  const [pointerX, pointerY] = point;
  const [tooltipWidth, tooltipHeight] = size.contentSize;
  const [viewWidth, viewHeight] = size.viewSize;
  const left = clamp(pointerX + gap, margin, viewWidth - tooltipWidth - margin);
  const preferredTop = pointerY - tooltipHeight - gap;
  const fallbackTop = pointerY + gap;
  const top = preferredTop >= margin ? preferredTop : fallbackTop;
  return [left, clamp(top, margin, viewHeight - tooltipHeight - margin)];
}

function clamp(value: number, min: number, max: number): number {
  if (max < min) return min;
  if (value < min) return min;
  if (value > max) return max;
  return value;
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
