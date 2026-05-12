import type { ForecastLayerState } from "../forecast-layer-matrix";
import type {
  ForecastChartPalette,
  ScenarioLine,
  TimelineEntry,
  VariableLine,
} from "./forecast-chart-types";

export function buildSeries(
  timeline: TimelineEntry[],
  separatorIndex: number | null,
  palette: ForecastChartPalette,
  scenarios: ScenarioLine[],
  variables: VariableLine[],
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
    ...scenarios.map((scenario, index) =>
      seriesLine(
        scenario.name,
        timeline.map((entry) => (
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
        timeline.map((entry) =>
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

function scenarioColor(palette: ForecastChartPalette, index: number): string {
  return palette.scenarios[index % Math.max(palette.scenarios.length, 1)] || palette.lineHistory;
}

function variableColor(palette: ForecastChartPalette, index: number): string {
  return palette.variables[index % Math.max(palette.variables.length, 1)] || palette.inkMuted;
}
