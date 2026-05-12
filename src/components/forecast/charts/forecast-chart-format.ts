import { formatForecastValue, type ForecastMetricMeta } from "../forecast-view-format";

interface TimelineEntry {
  fullLabel: string;
  historyValue: number | null;
  forecastValue: number | null;
  scenarioValues: { id: string; name: string; value: number }[];
  variableValues: { id: string; name: string; value: number; rawValue: number }[];
  lowerValue: number | null;
  upperValue: number | null;
}

interface ChartLabels {
  history: string;
  forecast: string;
  confidence: string;
}

export function formatAxisValue(value: number, locale: string, metric: ForecastMetricMeta): string {
  if (metric.unitKind === "currency-eur") {
    return new Intl.NumberFormat(locale, {
      notation: "compact",
      maximumFractionDigits: 1,
    }).format(value) + " €";
  }
  return new Intl.NumberFormat(locale, {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(value);
}

export function formatTooltip(
  raw: unknown,
  timeline: TimelineEntry[],
  metric: ForecastMetricMeta,
  labels: ChartLabels,
  locale: string
): string {
  const items = Array.isArray(raw) ? raw : [];
  const probe = items.find(hasDataIndex);
  const point = probe ? timeline[probe.dataIndex] : null;
  if (!point) return "";

  const lines = [`<div style="margin-bottom:6px;font-weight:600;">${point.fullLabel}</div>`];
  if (point.historyValue != null) {
    lines.push(`<div>${labels.history}: ${formatForecastValue(point.historyValue, locale, metric)}</div>`);
  }
  if (point.forecastValue != null) {
    lines.push(`<div>${labels.forecast}: ${formatForecastValue(point.forecastValue, locale, metric)}</div>`);
  }
  for (const scenario of point.scenarioValues) {
    lines.push(`<div>${scenario.name}: ${formatForecastValue(scenario.value, locale, metric)}</div>`);
  }
  for (const variable of point.variableValues) {
    lines.push(
      `<div>${variable.name}: ${new Intl.NumberFormat(locale, {
        minimumFractionDigits: 0,
        maximumFractionDigits: 2,
      }).format(variable.rawValue)}</div>`,
    );
  }
  if (point.lowerValue != null && point.upperValue != null) {
    const lower = formatForecastValue(point.lowerValue, locale, metric);
    const upper = formatForecastValue(point.upperValue, locale, metric);
    lines.push(`<div>${labels.confidence}: ${lower} - ${upper}</div>`);
  }
  return lines.join("");
}

function hasDataIndex(value: unknown): value is { dataIndex: number } {
  return (
    typeof value === "object" &&
    value !== null &&
    "dataIndex" in value &&
    typeof (value as { dataIndex?: unknown }).dataIndex === "number"
  );
}
