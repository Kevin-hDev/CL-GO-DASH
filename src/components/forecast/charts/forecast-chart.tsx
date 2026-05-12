import { useEffect, useRef } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { LineChart } from "echarts/charts";
import { DataZoomComponent, GridComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { ArrowsClockwise } from "@/components/ui/icons";
import { buildForecastChartOption } from "./forecast-chart-option";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import type { Point, ScenarioLine, VariableLine } from "./forecast-chart-types";
import { useForecastChartZoom } from "./use-forecast-chart-zoom";
import "./forecast-chart.css";

echarts.use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, DataZoomComponent]);

interface ForecastChartProps {
  history: Point[];
  predictions: Point[];
  scenarios: ScenarioLine[];
  variables: VariableLine[];
  quantiles: { q10: number[]; q90: number[] };
  frequency: string;
  endDate: string;
  locale: string;
  targetColumn?: string;
  fallbackName?: string;
  labels: { history: string; forecast: string; confidence: string };
  layers: ForecastLayerState;
}

export function ForecastChart(props: ForecastChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<EChartsType | null>(null);
  const propsRef = useRef(props);
  const zoomSignature = `${props.history.length}:${props.predictions.length}:${props.targetColumn ?? ""}:${props.fallbackName ?? ""}`;
const {
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
  } = useForecastChartZoom({ signature: zoomSignature, chartRef });
  const sliderValue = Math.round(100 - zoomSpan);

  useEffect(() => {
    propsRef.current = props;
  }, [props]);

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
        palette: {
          lineHistory: root.getPropertyValue("--fc-line-history").trim(),
          linePredict: root.getPropertyValue("--fc-line-predict").trim(),
          pointPredict: root.getPropertyValue("--fc-point-predict").trim(),
          band90: root.getPropertyValue("--fc-band-90").trim(),
          separator: root.getPropertyValue("--fc-separator").trim(),
          edge: root.getPropertyValue("--edge").trim(),
          inkMuted: root.getPropertyValue("--ink-faint").trim(),
          scenarios: scenarioPalette(root),
          variables: variablePalette(root),
        },
        zoomWindow,
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
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, [handleDataZoom, zoomWindow]);

  useEffect(() => {
    if (!chartRef.current || !containerRef.current) return;
    const root = getComputedStyle(containerRef.current);
    chartRef.current.setOption(buildForecastChartOption({
      ...props,
      palette: {
        lineHistory: root.getPropertyValue("--fc-line-history").trim(),
        linePredict: root.getPropertyValue("--fc-line-predict").trim(),
        pointPredict: root.getPropertyValue("--fc-point-predict").trim(),
        band90: root.getPropertyValue("--fc-band-90").trim(),
        separator: root.getPropertyValue("--fc-separator").trim(),
        edge: root.getPropertyValue("--edge").trim(),
        inkMuted: root.getPropertyValue("--ink-faint").trim(),
        scenarios: scenarioPalette(root),
        variables: variablePalette(root),
      },
      zoomWindow,
    }), true);
    chartRef.current.resize();
  }, [props, zoomWindow]);

  return (
    <div
      ref={shellRef}
      className={`fcc-chart-shell ${zoomSpan < 100 ? "is-draggable" : ""}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={stopDrag}
      onPointerLeave={stopDrag}
    >
      <div ref={containerRef} className="fcc-chart-root" />
      <div ref={controlsRef} className="fcc-chart-controls">
        <button className="fcc-chart-reset" type="button" onClick={handleResetZoom}>
          <ArrowsClockwise size={13} />
        </button>
        <input
          className="fcc-chart-zoom"
          type="range"
          min={0}
          max={85}
          step={1}
          value={sliderValue}
          onChange={(event) => handleZoomSlider(100 - Number(event.target.value))}
        />
      </div>
    </div>
  );
}

function scenarioPalette(root: CSSStyleDeclaration): string[] {
  return [
    root.getPropertyValue("--fc-scenario-a").trim(),
    root.getPropertyValue("--fc-scenario-b").trim(),
    root.getPropertyValue("--fc-scenario-c").trim(),
  ].filter(Boolean);
}

function variablePalette(root: CSSStyleDeclaration): string[] {
  return [
    root.getPropertyValue("--fc-variable-a").trim(),
    root.getPropertyValue("--fc-variable-b").trim(),
    root.getPropertyValue("--fc-variable-c").trim(),
    root.getPropertyValue("--fc-variable-d").trim(),
  ].filter(Boolean);
}
