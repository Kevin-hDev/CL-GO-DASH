export const FORECAST_CHART_MIN_ZOOM_SPAN = 10;

export interface ForecastZoomWindow {
  start: number;
  end: number;
}

export function clampForecastZoomWindow(start: number, end: number): ForecastZoomWindow {
  if (!Number.isFinite(start) || !Number.isFinite(end)) return { start: 0, end: 100 };
  const rawStart = Math.min(start, end);
  const rawEnd = Math.max(start, end);
  const span = Math.min(100, Math.max(FORECAST_CHART_MIN_ZOOM_SPAN, rawEnd - rawStart));
  if (span >= 100) return { start: 0, end: 100 };

  const center = (rawStart + rawEnd) / 2;
  const nextStart = center - span / 2;
  const nextEnd = center + span / 2;
  if (nextStart < 0) return { start: 0, end: span };
  if (nextEnd > 100) return { start: 100 - span, end: 100 };
  return { start: nextStart, end: nextEnd };
}

export function sameForecastZoomWindow(left: ForecastZoomWindow, right: ForecastZoomWindow): boolean {
  return Math.abs(left.start - right.start) < 0.001 && Math.abs(left.end - right.end) < 0.001;
}

export function forecastZoomSliderValue(span: number): number {
  const max = 100 - FORECAST_CHART_MIN_ZOOM_SPAN;
  if (!Number.isFinite(span)) return 0;
  return Math.max(0, Math.min(max, Math.round(100 - span)));
}

export function buildForecastZoomSignature(props: {
  history: { date: string }[];
  predictions: { date: string }[];
  targetColumn?: string;
  fallbackName?: string;
}): string {
  const first = props.history[0]?.date ?? props.predictions[0]?.date ?? "";
  const lastHistory = props.history[props.history.length - 1]?.date ?? "";
  const lastPrediction = props.predictions[props.predictions.length - 1]?.date ?? "";
  return [
    first,
    lastHistory,
    lastPrediction,
    props.history.length,
    props.predictions.length,
    props.targetColumn ?? "",
    props.fallbackName ?? "",
  ].join(":");
}
