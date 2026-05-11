import { useCallback, useRef, useState } from "react";

const DEFAULT_CHART_HEIGHT = 270;
const MIN_CHART_HEIGHT = 200;

export function useForecastChartResize() {
  const [chartHeight, setChartHeight] = useState(DEFAULT_CHART_HEIGHT);
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
        const maxHeight = Math.floor(window.innerHeight * 0.6);
        setChartHeight(
          Math.max(
            MIN_CHART_HEIGHT,
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
    [chartHeight]
  );

  const resetHeight = useCallback(() => {
    setChartHeight(DEFAULT_CHART_HEIGHT);
  }, []);

  return { chartHeight, isResizing, startResize, resetHeight };
}
