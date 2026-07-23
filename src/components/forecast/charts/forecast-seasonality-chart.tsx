import { useEffect, useRef } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { LineChart } from "echarts/charts";
import {
  GridComponent,
  MarkLineComponent,
  TooltipComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { useTranslation } from "react-i18next";
import {
  filterForecastSeriesData,
  type ForecastViewResult,
} from "../sections/forecast-view-data";
import { useForecastResult } from "../use-forecast-result";
import { buildForecastChartPalette } from "./forecast-chart-palette";
import {
  buildSeasonalityModel,
  type SeasonalityModel,
} from "./forecast-seasonality-data";
import { buildSeasonalityOption } from "./forecast-seasonality-option";

echarts.use([
  CanvasRenderer,
  LineChart,
  GridComponent,
  TooltipComponent,
  MarkLineComponent,
]);

interface ForecastSeasonalityChartProps {
  analysisId: string;
  /** Bump to request a resize after the card expand transition. */
  resizeSignal?: number;
}

export function ForecastSeasonalityChart({
  analysisId,
  resizeSignal = 0,
}: ForecastSeasonalityChartProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastViewResult>(
    analysisId,
    t("forecast.noAnalysis"),
  );
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<EChartsType | null>(null);
  const lastModelRef = useRef<SeasonalityModel | null>(null);

  const model = data
    ? buildSeasonalityModel(
        filterForecastSeriesData(data, data.input_data.series_ids?.[0] ?? "", [])
          .history,
        i18n.language,
      )
    : null;

  const applyOptionRef = useRef((_replace: boolean) => {});
  applyOptionRef.current = (replace: boolean) => {
    if (!chartRef.current || !containerRef.current || !model) return;
    const root = getComputedStyle(containerRef.current);
    chartRef.current.setOption(
      buildSeasonalityOption(model, buildForecastChartPalette(root), {
        indexBase: t("forecast.companion.indexBase"),
      }),
      replace,
    );
  };

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return undefined;
    const ensureChart = () => {
      if (chartRef.current) return;
      if (container.clientWidth <= 0 || container.clientHeight <= 0) return;
      chartRef.current = echarts.init(container, undefined, { renderer: "canvas" });
      applyOptionRef.current(true);
    };
    const observer = new ResizeObserver(() => {
      if (!chartRef.current) ensureChart();
      else chartRef.current.resize();
    });
    observer.observe(container);
    ensureChart();
    return () => {
      observer.disconnect();
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, []);

  useEffect(() => {
    const replace = lastModelRef.current !== model;
    lastModelRef.current = model;
    applyOptionRef.current(replace);
  });

  useEffect(() => {
    if (resizeSignal > 0) chartRef.current?.resize();
  }, [resizeSignal]);

  return (
    <div className="fcwf-companion fcwf-companion-seasonality">
      <div ref={containerRef} className="fcwf-companion-chart" />
      {error ? (
        <div className="fcwf-companion-empty">{error}</div>
      ) : !model ? (
        <div className="fcwf-companion-empty">
          {t("forecast.companion.insufficientData")}
        </div>
      ) : null}
    </div>
  );
}
