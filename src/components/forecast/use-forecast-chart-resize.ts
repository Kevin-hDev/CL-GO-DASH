import { useCallback, useRef, useState } from "react";

const DEFAULT_CHART_HEIGHT = 270;
const MIN_CHART_HEIGHT = 200;

interface ResizeOptions {
  defaultHeight?: number;
  minHeight?: number;
  maxWindowRatio?: number;
}

export function useForecastChartResize(options: ResizeOptions = {}) {
  const defaultHeight = options.defaultHeight ?? DEFAULT_CHART_HEIGHT;
  const minHeight = options.minHeight ?? MIN_CHART_HEIGHT;
  const maxWindowRatio = options.maxWindowRatio ?? 0.6;
  const [chartHeight, setChartHeight] = useState(defaultHeight);
  const [isResizing, setIsResizing] = useState(false);
  const resizeRef = useRef<{ startY: number; startHeight: number } | null>(null);

  const startResize = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      resizeRef.current = {
        startY: event.clientY,
        startHeight: chartHeight,
      };
      setIsResizing(true);

      const onMove = (moveEvent: PointerEvent) => {
        if (!resizeRef.current) return;
        const delta = moveEvent.clientY - resizeRef.current.startY;
        const maxHeight = Math.floor(window.innerHeight * maxWindowRatio);
        setChartHeight(
          Math.max(
            minHeight,
            Math.min(maxHeight, resizeRef.current.startHeight + delta)
          )
        );
      };

      const onUp = () => {
        resizeRef.current = null;
        setIsResizing(false);
        window.removeEventListener("pointermove", onMove);
        window.removeEventListener("pointerup", onUp);
      };

      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
    },
    [chartHeight, maxWindowRatio, minHeight]
  );

  const resetHeight = useCallback(() => {
    setChartHeight(defaultHeight);
  }, [defaultHeight]);

  return { chartHeight, isResizing, startResize, resetHeight };
}
