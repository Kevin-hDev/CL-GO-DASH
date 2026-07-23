import type { EChartsType } from "echarts/core";
import { useEffect } from "react";
import {
  computeWheelZoomWindow,
  sameForecastZoomWindow,
  zoomAnchorRatio,
  FORECAST_CHART_MIN_ZOOM_SPAN,
  type ForecastZoomWindow,
} from "./forecast-chart-zoom-utils";

interface UseForecastWheelZoomArgs {
  shellRef: React.RefObject<HTMLDivElement | null>;
  chartRef: React.RefObject<EChartsType | null>;
  signature: string;
  syncedRef: React.RefObject<{ signature: string; start: number; end: number }>;
  syncZoomState: (window: ForecastZoomWindow) => void;
}

function readChartGridRect(chart: EChartsType): { x: number; width: number } | null {
  // getModel is typed private; it is the pragmatic way to read the exact
  // plot rect (containLabel-aware) for the wheel anchor.
  const model = (chart as unknown as { getModel: () => unknown }).getModel() as {
    getComponent: (
      type: string,
      index: number,
    ) => { coordinateSystem?: { getRect?: () => { x: number; width: number } } } | undefined;
  };
  const rect = model.getComponent("grid", 0)?.coordinateSystem?.getRect?.();
  return rect && Number.isFinite(rect.x) && rect.width > 0 ? rect : null;
}

// Owns the wheel zoom pipeline: ECharts inside-roam wheel handling is
// disabled (zoomOnMouseWheel: false) because its cursor-anchored zoom-in
// contracts the window toward the cursor and reads as a silent pan, and
// patching it event-by-event proved unreliable. Here every wheel tick
// computes the next window with computeWheelZoomWindow and applies it via
// the same dispatch + syncZoomState path used by the slider and jump bars.
export function useForecastWheelZoom({
  shellRef,
  chartRef,
  signature,
  syncedRef,
  syncZoomState,
}: UseForecastWheelZoomArgs) {
  useEffect(() => {
    const shell = shellRef.current;
    if (!shell) return undefined;
    const onWheel = (event: WheelEvent) => {
      // Never let the wheel scroll the page/panel while over the chart.
      event.preventDefault();
      if (event.deltaY === 0) return;
      const chart = chartRef.current;
      if (!chart) return;
      const synced = syncedRef.current;
      const current = synced.signature === signature
        ? { start: synced.start, end: synced.end }
        : { start: 0, end: 100 };
      const dom = chart.getDom();
      const rect = dom.getBoundingClientRect();
      const gridRect = readChartGridRect(chart);
      const anchor = zoomAnchorRatio(
        event.clientX - rect.left,
        gridRect?.x ?? 0,
        gridRect?.width ?? rect.width,
      );
      const next = computeWheelZoomWindow(
        current,
        event.deltaY > 0 ? 1 : -1,
        anchor,
        FORECAST_CHART_MIN_ZOOM_SPAN,
      );
      if (sameForecastZoomWindow(current, next)) return;
      chart.dispatchAction({ type: "dataZoom", ...next });
      syncZoomState(next);
    };
    // Capture + non-passive: runs before zrender's bubble-phase listener and
    // is allowed to preventDefault (React onWheel cannot).
    shell.addEventListener("wheel", onWheel, { passive: false, capture: true });
    return () => shell.removeEventListener("wheel", onWheel, { capture: true });
  }, [shellRef, chartRef, signature, syncedRef, syncZoomState]);
}
