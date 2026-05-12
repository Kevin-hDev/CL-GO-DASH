import type { EChartsType } from "echarts/core";
import { useCallback, useMemo, useRef, useState } from "react";

interface UseForecastChartZoomArgs {
  signature: string;
  chartRef: React.RefObject<EChartsType | null>;
}

export function useForecastChartZoom({
  signature,
  chartRef,
}: UseForecastChartZoomArgs) {
  const shellRef = useRef<HTMLDivElement | null>(null);
  const controlsRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef({ active: false, x: 0, start: 0, end: 100 });
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

  const handleDataZoom = useCallback(() => {
    const option = chartRef.current?.getOption() as
      | { dataZoom?: Array<{ start?: number; end?: number }> }
      | undefined;
    const zoom = option?.dataZoom?.[0];
    if (typeof zoom?.start !== "number" || typeof zoom?.end !== "number") return;
    const nextStart = zoom.start;
    const nextEnd = zoom.end;
    setZoomState((current) =>
      current.signature === signature &&
      current.start === nextStart &&
      current.end === nextEnd
        ? current
        : { signature, start: nextStart, end: nextEnd },
    );
  }, [chartRef, signature]);

  const handleResetZoom = useCallback(() => {
    setZoomState({ signature, start: 0, end: 100 });
  }, [signature]);

  const handleZoomSlider = useCallback((nextSpan: number) => {
    setZoomState((current) => {
      const center =
        current.signature === signature ? (current.start + current.end) / 2 : 50;
      const next = clampWindow(center - nextSpan / 2, center + nextSpan / 2);
      return { signature, ...next };
    });
  }, [signature]);

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
    const shift = (-dx / width) * span;
    setZoomState({
      signature,
      ...clampWindow(dragRef.current.start + shift, dragRef.current.end + shift),
    });
  }, [signature]);

  const stopDrag = useCallback((event?: React.PointerEvent<HTMLDivElement>) => {
    if (event && event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    dragRef.current.active = false;
  }, []);

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

function clampWindow(start: number, end: number) {
  const span = Math.max(1, end - start);
  if (span >= 100) return { start: 0, end: 100 };
  if (start < 0) return { start: 0, end: span };
  if (end > 100) return { start: 100 - span, end: 100 };
  return { start, end };
}
