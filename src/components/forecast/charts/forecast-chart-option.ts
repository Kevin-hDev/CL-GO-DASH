import type { EChartsOption } from "echarts";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import { buildPeriodMeta, inferMetricMeta, type ForecastMetricMeta } from "../forecast-view-format";
import { formatAxisValue, formatTooltip } from "./forecast-chart-format";

interface Point {
  date: string;
  value: number;
}

interface TimelineEntry {
  axisLabel: string;
  fullLabel: string;
  historyValue: number | null;
  forecastValue: number | null;
  lowerValue: number | null;
  upperValue: number | null;
}

export interface ForecastChartOptionArgs {
  history: Point[];
  predictions: Point[];
  quantiles: { q10: number[]; q90: number[] };
  frequency: string;
  endDate: string;
  locale: string;
  targetColumn?: string;
  fallbackName?: string;
  layers: ForecastLayerState;
  palette: {
    lineHistory: string;
    linePredict: string;
    pointPredict: string;
    band90: string;
    separator: string;
    edge: string;
    inkMuted: string;
  };
  labels: { history: string; forecast: string; confidence: string };
}

export function buildForecastChartOption(args: ForecastChartOptionArgs): EChartsOption {
  const metric = inferMetricMeta(args.locale, args.targetColumn, args.fallbackName);
  const timeline = buildTimeline(args);
  const separatorIndex = args.history.length > 0 ? args.history.length - 1 : null;

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
    yAxis: buildYAxis(args.locale, metric, args.palette.edge, args.palette.inkMuted),
    series: buildSeries(
      timeline,
      separatorIndex,
      args.palette,
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

function buildYAxis(locale: string, metric: ForecastMetricMeta, edge: string, inkMuted: string) {
  return {
    type: "value" as const,
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

function buildSeries(
  timeline: TimelineEntry[],
  separatorIndex: number | null,
  palette: ForecastChartOptionArgs["palette"],
  confidenceLabel: string,
  layers: ForecastLayerState
) {
  const lower = timeline.map((entry) =>
    layers.confidence ? entry.lowerValue : null
  );
  const band = timeline.map((entry) =>
    layers.confidence && entry.lowerValue != null && entry.upperValue != null
      ? entry.upperValue - entry.lowerValue
      : null
  );

  return [
    seriesLine(confidenceLabel, lower, { opacity: 0 }, undefined, "confidence-90", false),
    { ...seriesLine(confidenceLabel, band, { opacity: 0 }, palette.band90, "confidence-90", false), areaStyle: { color: palette.band90 } },
    seriesLine(
      "history",
      timeline.map((entry) => (layers.history ? entry.historyValue : null)),
      { color: palette.lineHistory, width: 2 },
      undefined,
      undefined,
      false
    ),
    {
      ...seriesLine(
        "forecast",
        timeline.map((entry) => (layers.forecast ? entry.forecastValue : null)),
        { color: palette.linePredict, width: 1.8 },
        undefined,
        undefined,
        true
      ),
      itemStyle: { color: palette.pointPredict, borderColor: palette.linePredict, borderWidth: 2 },
      markLine: separatorIndex != null && layers.forecast ? {
        symbol: ["none", "none"],
        silent: true,
        label: { show: false },
        lineStyle: { color: palette.separator, type: "dashed" as const, width: 1 },
        data: [{ xAxis: separatorIndex }],
      } : undefined,
    },
  ];
}

function seriesLine(
  name: string,
  data: Array<number | null>,
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
    connectNulls: false,
    lineStyle,
    areaStyle: areaColor ? { color: areaColor } : undefined,
    emphasis: { disabled: true },
  };
}

function buildTimeline(args: ForecastChartOptionArgs): TimelineEntry[] {
  return [
    ...args.history.map((point) => ({
      axisLabel: shortDate(point.date, args.locale),
      fullLabel: point.date,
      historyValue: point.value,
      forecastValue: null,
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
