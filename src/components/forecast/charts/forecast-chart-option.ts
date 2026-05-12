import type { EChartsOption } from "echarts";
import { buildPeriodMeta, inferMetricMeta, type ForecastMetricMeta } from "../forecast-view-format";
import { formatAxisValue, formatTooltip } from "./forecast-chart-format";
import { buildSeries } from "./forecast-chart-series";
import type { ForecastChartOptionArgs, TimelineEntry } from "./forecast-chart-types";

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
      args.scenarios,
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

function buildTimeline(args: ForecastChartOptionArgs): TimelineEntry[] {
  return [
    ...args.history.map((point) => ({
      axisLabel: shortDate(point.date, args.locale),
      fullLabel: point.date,
      historyValue: point.value,
      forecastValue: null,
      scenarioValues: [],
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
          .filter((scenario): scenario is { id: string; name: string; value: number } =>
            typeof scenario.value === "number"
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
