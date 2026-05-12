import { useEffect, useRef } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { LineChart } from "echarts/charts";
import { GridComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { buildForecastChartOption } from "./forecast-chart-option";
import type { ForecastLayerState } from "../forecast-layer-matrix";
import type { Point, ScenarioLine } from "./forecast-chart-types";
import "./forecast-chart.css";

echarts.use([CanvasRenderer, LineChart, GridComponent, TooltipComponent]);

interface ForecastChartProps {
  history: Point[];
  predictions: Point[];
  scenarios: ScenarioLine[];
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
        },
      }), true);
    };

    const ensureChart = () => {
      if (disposed || chartRef.current || !containerRef.current) return;
      if (containerRef.current.clientWidth <= 0 || containerRef.current.clientHeight <= 0) {
        frameId = requestAnimationFrame(ensureChart);
        return;
      }
      chartRef.current = echarts.init(containerRef.current, undefined, { renderer: "canvas" });
      applyOptions();
      chartRef.current.resize();
    };

    observer.observe(container);
    ensureChart();

    return () => {
      disposed = true;
      if (frameId) cancelAnimationFrame(frameId);
      observer.disconnect();
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, []);

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
      },
    }), true);
    chartRef.current.resize();
  }, [props]);

  return <div ref={containerRef} className="fcc-chart-root" />;
}

function scenarioPalette(root: CSSStyleDeclaration): string[] {
  return [
    root.getPropertyValue("--fc-scenario-a").trim(),
    root.getPropertyValue("--fc-scenario-b").trim(),
    root.getPropertyValue("--fc-scenario-c").trim(),
  ].filter(Boolean);
}
