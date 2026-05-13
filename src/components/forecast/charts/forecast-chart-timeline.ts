import { buildPeriodMeta } from "../forecast-view-format";
import type { ForecastChartAnnotation, ForecastChartOptionArgs, TimelineEntry } from "./forecast-chart-types";

export function buildTimeline(args: ForecastChartOptionArgs): TimelineEntry[] {
  const annotationsByDate = groupAnnotations(args.annotations);
  return [
    ...args.history.map((point, historyIndex) => ({
      axisLabel: shortDate(point.date, args.locale),
      dateKey: dateKey(point.date),
      timestamp: timestamp(point.date),
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
        .filter(hasVariableValue),
      annotationValues: annotationsByDate.get(dateKey(point.date)) ?? [],
      lowerValue: null,
      upperValue: null,
    })),
    ...args.predictions.map((point, index) => {
      const period = buildPeriodMeta(index, point.date, args.endDate, args.frequency, args.locale);
      return {
        axisLabel: shortDate(point.date, args.locale),
        dateKey: dateKey(point.date),
        timestamp: timestamp(point.date),
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
          .filter((variable): variable is { id: string; name: string; value: number; rawValue: number } =>
            args.layers[variable.id] === true &&
            typeof variable.value === "number" && typeof variable.rawValue === "number",
          ),
        annotationValues: annotationsByDate.get(dateKey(point.date)) ?? [],
        lowerValue: args.quantiles.q10[index] ?? null,
        upperValue: args.quantiles.q90[index] ?? null,
      };
    }),
  ];
}

function groupAnnotations(annotations: ForecastChartAnnotation[]) {
  const map = new Map<string, ForecastChartAnnotation[]>();
  for (const annotation of annotations) {
    const key = dateKey(annotation.date);
    map.set(key, [...(map.get(key) ?? []), annotation]);
  }
  return map;
}

function hasVariableValue(
  variable: { id: string; name: string; value: number | null; rawValue: number | null },
): variable is { id: string; name: string; value: number; rawValue: number } {
  return typeof variable.value === "number" && typeof variable.rawValue === "number";
}

function shortDate(value: string, locale: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat(locale, { month: "2-digit", day: "2-digit" }).format(parsed);
}

function dateKey(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value.slice(0, 10);
  return parsed.toISOString().slice(0, 10);
}

function timestamp(value: string): number {
  const parsed = new Date(value).getTime();
  if (!Number.isNaN(parsed)) return parsed;
  const dateOnly = new Date(`${value.slice(0, 10)}T00:00:00`).getTime();
  return Number.isNaN(dateOnly) ? 0 : dateOnly;
}
