import { useCallback, useEffect, useRef } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { LineChart, ScatterChart } from "echarts/charts";
import { DataZoomComponent, GridComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { ArrowsClockwise } from "@/components/ui/icons";
import {
  dateKey,
  readDataIndex,
  readFirstAnnotationId,
  readSeriesId,
} from "./forecast-chart-events";
import { buildForecastChartOption } from "./forecast-chart-option";
import { buildForecastChartPalette } from "./forecast-chart-palette";
import type { ForecastChartProps } from "./forecast-chart-types";
import {
  forecastZoomSliderValue,
  FORECAST_CHART_MIN_ZOOM_SPAN,
} from "./forecast-chart-zoom-utils";
import { useForecastChartZoom } from "./use-forecast-chart-zoom";
import "./forecast-chart.css";

echarts.use([
  CanvasRenderer,
  LineChart,
  ScatterChart,
  GridComponent,
  TooltipComponent,
  DataZoomComponent,
]);

export function ForecastChart(props: ForecastChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<EChartsType | null>(null);
  const propsRef = useRef(props);
  const zoomWindowRef = useRef({ start: 0, end: 100 });
  const zoomSignature = buildZoomSignature(props);
  const handleZoomRefChange = useCallback((window: { start: number; end: number }) => {
    zoomWindowRef.current = window;
  }, []);
  const {
    shellRef,
    controlsRef,
    zoomSpan,
    handleDataZoom,
    handleResetZoom,
    handleZoomSlider,
    handlePointerDown,
    handlePointerMove,
    stopDrag,
  } = useForecastChartZoom({
    signature: zoomSignature,
    chartRef,
    onZoomChange: handleZoomRefChange,
  });
  const sliderValue = forecastZoomSliderValue(zoomSpan);

  useEffect(() => {
    propsRef.current = props;
  }, [props]);

  const handleChartClick = useCallback((event: unknown) => {
    const dataIndex = readDataIndex(event);
    if (dataIndex == null) return;
    const source = readSeriesId(event);
    if (source !== "annotation-user" && source !== "annotation-llm") return;
    const explicitId = readFirstAnnotationId(event);
    const annotation = explicitId
      ? propsRef.current.annotations?.find((item) => item.id === explicitId)
      : propsRef.current.annotations?.find((item) => {
          const key = dateKey(item.date);
          const points = [...propsRef.current.history, ...propsRef.current.predictions];
          return points[dataIndex] && dateKey(points[dataIndex].date) === key;
        });
    if (annotation) propsRef.current.onAnnotationClick?.(annotation);
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    let disposed = false;
    let frameId = 0;
    const observer = new ResizeObserver(() => {
      if (chartRef.current) {
        chartRef.current.resize();
        return;
      }
      ensureChart();
    });

    const applyOptions = () => {
      if (!chartRef.current || !containerRef.current) return;
      const root = getComputedStyle(containerRef.current);
      chartRef.current.setOption(buildForecastChartOption({
        ...propsRef.current,
        palette: buildForecastChartPalette(root),
        annotations: propsRef.current.annotations ?? [],
        compact: Boolean(propsRef.current.compact),
        zoomWindow: zoomWindowRef.current,
      }), true);
    };

    const ensureChart = () => {
      if (disposed || chartRef.current || !containerRef.current) return;
      if (containerRef.current.clientWidth <= 0 || containerRef.current.clientHeight <= 0) {
        frameId = requestAnimationFrame(ensureChart);
        return;
      }
      chartRef.current = echarts.init(containerRef.current, undefined, { renderer: "canvas" });
      chartRef.current.on("datazoom", handleDataZoom);
      chartRef.current.on("click", handleChartClick);
      applyOptions();
      chartRef.current.resize();
    };

    observer.observe(container);
    ensureChart();

    return () => {
      disposed = true;
      if (frameId) cancelAnimationFrame(frameId);
      observer.disconnect();
      chartRef.current?.off("datazoom", handleDataZoom);
      chartRef.current?.off("click", handleChartClick);
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, [handleChartClick, handleDataZoom]);

  useEffect(() => {
    if (!chartRef.current || !containerRef.current) return;
    const root = getComputedStyle(containerRef.current);
    chartRef.current.setOption(buildForecastChartOption({
      ...props,
      palette: buildForecastChartPalette(root),
      annotations: props.annotations ?? [],
      compact: Boolean(props.compact),
      zoomWindow: zoomWindowRef.current,
    }), true);
    chartRef.current.resize();
  }, [props]);

  return (
    <div
      ref={shellRef}
      className={`fcc-chart-shell fcc-chart-${props.mode ?? "main"} ${props.compact ? "is-compact" : ""} ${zoomSpan < 100 ? "is-draggable" : ""}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={stopDrag}
      onPointerLeave={stopDrag}
    >
      <div ref={containerRef} className="fcc-chart-root" />
      {!props.compact && (
        <div ref={controlsRef} className="fcc-chart-controls">
          <button className="fcc-chart-reset" type="button" onClick={handleResetZoom}>
            <ArrowsClockwise size={13} />
          </button>
          <input
            className="fcc-chart-zoom"
            type="range"
            min={0}
            max={100 - FORECAST_CHART_MIN_ZOOM_SPAN}
            step={1}
            value={sliderValue}
            onChange={(event) => handleZoomSlider(100 - Number(event.target.value))}
          />
        </div>
      )}
    </div>
  );
}

function buildZoomSignature(props: ForecastChartProps): string {
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
