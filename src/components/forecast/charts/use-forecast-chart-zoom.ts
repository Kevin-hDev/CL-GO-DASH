import type { EChartsType } from "echarts/core";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  clampForecastZoomWindow,
  FORECAST_CHART_MIN_ZOOM_SPAN,
  sameForecastZoomWindow,
  shouldIgnoreRoamAtFullExtent,
  type ForecastZoomWindow,
} from "./forecast-chart-zoom-utils";

interface UseForecastChartZoomArgs {
  signature: string;
  chartRef: React.RefObject<EChartsType | null>;
  onZoomChange: (window: ForecastZoomWindow) => void;
  jump?: { start: number; seq: number } | null;
}

export function useForecastChartZoom({
  signature,
  chartRef,
  onZoomChange,
  jump,
}: UseForecastChartZoomArgs) {
  const shellRef = useRef<HTMLDivElement | null>(null);
  const controlsRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef({ active: false, x: 0, start: 0, end: 100 });
  // Last window synced to React state; lets handleDataZoom know the window
  // before a roam gesture without depending on re-renders.
  const syncedRef = useRef({ signature, start: 0, end: 100 });
  const [zoomState, setZoomState] = useState({
    signature,
    start: 0,
    end: 100,
  });

  const zoomWindow = useMemo(
    () =>
      zoomState.signature === signature
        ? { start: zoomState.start, end: zoomState.end }
        : { start: 0, end: 100 },
    [signature, zoomState.end, zoomState.signature, zoomState.start],
  );
  const zoomSpan = useMemo(() => zoomWindow.end - zoomWindow.start, [zoomWindow]);

  const readChartZoom = useCallback(() => {
    const option = chartRef.current?.getOption() as
      | { dataZoom?: Array<{ start?: number; end?: number }> }
      | undefined;
    const zoom = option?.dataZoom?.[0];
    if (typeof zoom?.start !== "number" || typeof zoom?.end !== "number") return null;
    return { start: zoom.start, end: zoom.end };
  }, [chartRef]);

  const syncZoomState = useCallback((nextWindow: ForecastZoomWindow) => {
    const next = clampForecastZoomWindow(nextWindow.start, nextWindow.end);
    syncedRef.current = { signature, ...next };
    setZoomState((current) =>
      current.signature === signature &&
      current.start === next.start &&
      current.end === next.end
        ? current
        : { signature, ...next },
    );
    onZoomChange(next);
  }, [onZoomChange, signature]);

  const lastJumpSeqRef = useRef(0);
  useEffect(() => {
    if (!jump || jump.seq === lastJumpSeqRef.current) return;
    lastJumpSeqRef.current = jump.seq;
    const span = zoomWindow.end - zoomWindow.start;
    const next = clampForecastZoomWindow(jump.start, jump.start + span);
    chartRef.current?.dispatchAction({ type: "dataZoom", ...next });
    syncZoomState(next);
  }, [jump, zoomWindow, chartRef, syncZoomState]);

  const handleDataZoom = useCallback((event?: unknown) => {
    if (dragRef.current.active) return;
    const raw = readChartZoom();
    if (!raw) return;
    const next = clampForecastZoomWindow(raw.start, raw.end);
    const synced = syncedRef.current;
    const current = synced.signature === signature
      ? { start: synced.start, end: synced.end }
      : { start: 0, end: 100 };
    // Wheel/roam events carry `batch`; our own dispatchAction calls do not.
    // At full extent the wheel must be a complete no-op: ECharts anchors
    // wheel zoom-in at the cursor, which contracts the window toward it and
    // reads as a silent pan. Revert and leave state untouched. Our controls
    // (slider/jump/reset/drag) sync state directly, so they are unaffected.
    const isRoam = Array.isArray((event as { batch?: unknown } | undefined)?.batch);
    if (isRoam && shouldIgnoreRoamAtFullExtent(current, next)) {
      chartRef.current?.dispatchAction({ type: "dataZoom", start: 0, end: 100 });
      return;
    }
    if (!sameForecastZoomWindow(raw, next)) {
      chartRef.current?.dispatchAction({ type: "dataZoom", ...next });
    }
    syncZoomState(next);
  }, [chartRef, readChartZoom, signature, syncZoomState]);

  const handleResetZoom = useCallback(() => {
    chartRef.current?.dispatchAction({ type: "dataZoom", start: 0, end: 100 });
    syncZoomState({ start: 0, end: 100 });
  }, [chartRef, syncZoomState]);

  const handleZoomSlider = useCallback((nextSpan: number) => {
    const synced = syncedRef.current;
    const center =
      synced.signature === signature ? (synced.start + synced.end) / 2 : 50;
    const span = Math.max(FORECAST_CHART_MIN_ZOOM_SPAN, Math.min(100, nextSpan));
    const next = clampForecastZoomWindow(center - span / 2, center + span / 2);
    chartRef.current?.dispatchAction({ type: "dataZoom", ...next });
    syncZoomState(next);
  }, [chartRef, signature, syncZoomState]);

  const handlePointerDown = useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (zoomSpan >= 100 || controlsRef.current?.contains(event.target as Node)) return;
    dragRef.current = {
      active: true,
      x: event.clientX,
      start: zoomWindow.start,
      end: zoomWindow.end,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  }, [zoomSpan, zoomWindow.end, zoomWindow.start]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current.active || !shellRef.current) return;
    const width = shellRef.current.clientWidth || 1;
    const dx = event.clientX - dragRef.current.x;
    const span = dragRef.current.end - dragRef.current.start;
    const next = clampForecastZoomWindow(
      dragRef.current.start + (-dx / width) * span,
      dragRef.current.end + (-dx / width) * span,
    );
    chartRef.current?.dispatchAction({ type: "dataZoom", ...next });
  }, [chartRef]);

  const stopDrag = useCallback((event?: React.PointerEvent<HTMLDivElement>) => {
    if (event && event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    const raw = readChartZoom();
    if (raw) syncZoomState(raw);
    dragRef.current.active = false;
  }, [readChartZoom, syncZoomState]);

  return {
    shellRef,
    controlsRef,
    zoomWindow,
    zoomSpan,
    handleDataZoom,
    handleResetZoom,
    handleZoomSlider,
    handlePointerDown,
    handlePointerMove,
    stopDrag,
  };
}
