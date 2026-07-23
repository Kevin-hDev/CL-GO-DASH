import type { EChartsOption } from "echarts";
import type { SeasonalityModel } from "./forecast-seasonality-data";
import type { ForecastChartPalette } from "./forecast-chart-types";

export function buildSeasonalityOption(
  model: SeasonalityModel,
  palette: ForecastChartPalette,
  labels: { indexBase: string },
): EChartsOption {
  const rotation = [...palette.scenarios, ...palette.variables];
  const fallback = palette.inkMuted;

  return {
    animation: false,
    grid: { left: 12, right: 16, top: 34, bottom: 24, containLabel: true },
    tooltip: {
      trigger: "axis",
      backgroundColor: palette.tooltipBg,
      borderColor: palette.edge,
      borderWidth: 1,
      padding: [7, 10],
      textStyle: { color: palette.tooltipText, fontFamily: "var(--font-sans)", fontSize: 12, lineHeight: 17 },
      confine: true,
    },
    xAxis: {
      type: "category",
      data: model.months,
      boundaryGap: false,
      axisLine: { lineStyle: { color: palette.edge } },
      axisTick: { show: false },
      axisLabel: { color: palette.inkMuted, fontSize: 11 },
    },
    yAxis: {
      type: "value",
      scale: true,
      name: labels.indexBase,
      nameTextStyle: { color: palette.inkMuted, fontSize: 11, align: "left" },
      splitLine: { lineStyle: { color: palette.edge, opacity: 0.5 } },
      axisLine: { show: false },
      axisTick: { show: false },
      axisLabel: { color: palette.inkMuted, fontSize: 11 },
    },
    series: model.years.map((year, index) => ({
      name: String(year.year),
      type: "line" as const,
      data: year.values,
      smooth: 0.25,
      connectNulls: false,
      symbol: "circle" as const,
      showSymbol: false,
      symbolSize: 4,
      z: year.emphasized ? 4 : 2,
      lineStyle: {
        color: year.emphasized
          ? palette.linePredict
          : rotation[index % Math.max(rotation.length, 1)] || fallback,
        width: year.emphasized ? 2.2 : 1.5,
      },
      emphasis: { disabled: true },
      markLine: year.emphasized ? {
        symbol: ["none", "none"],
        silent: true,
        label: { show: false },
        lineStyle: { color: palette.separator, type: "dashed" as const, width: 1 },
        data: [{ yAxis: 100 }],
      } : undefined,
    })),
  };
}
