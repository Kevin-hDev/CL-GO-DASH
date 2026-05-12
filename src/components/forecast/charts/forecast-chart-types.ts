import type { ForecastLayerState } from "../forecast-layer-matrix";

export interface Point {
  date: string;
  value: number;
}

export interface ScenarioLine {
  id: string;
  name: string;
  predictions: Point[];
}

export interface VariableLine {
  id: string;
  name: string;
  values: Array<number | null>;
  rawValues: Array<number | null>;
}

export interface TimelineEntry {
  axisLabel: string;
  fullLabel: string;
  historyValue: number | null;
  forecastValue: number | null;
  scenarioValues: { id: string; name: string; value: number }[];
  variableValues: { id: string; name: string; value: number; rawValue: number }[];
  lowerValue: number | null;
  upperValue: number | null;
}

export interface ForecastChartPalette {
  lineHistory: string;
  linePredict: string;
  pointPredict: string;
  band90: string;
  separator: string;
  edge: string;
  inkMuted: string;
  scenarios: string[];
  variables: string[];
}

export interface ForecastChartOptionArgs {
  history: Point[];
  predictions: Point[];
  scenarios: ScenarioLine[];
  variables: VariableLine[];
  zoomWindow: { start: number; end: number };
  quantiles: { q10: number[]; q90: number[] };
  frequency: string;
  endDate: string;
  locale: string;
  targetColumn?: string;
  fallbackName?: string;
  layers: ForecastLayerState;
  palette: ForecastChartPalette;
  labels: { history: string; forecast: string; confidence: string };
}
