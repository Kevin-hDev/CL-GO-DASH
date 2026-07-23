import { useTranslation } from "react-i18next";
import {
  filterForecastSeriesData,
  type ForecastViewResult,
} from "../sections/forecast-view-data";
import { useForecastResult } from "../use-forecast-result";
import { ForecastChart } from "./forecast-chart";

const FAN_HISTORY_WINDOW = 18;
const FAN_LAYERS = { history: true, forecast: true, confidence: true };

interface ForecastFanChartProps {
  analysisId: string;
  /** Bumped by the card after an expand transition completes. */
  resizeSignal?: number;
}

export function ForecastFanChart({
  analysisId,
  resizeSignal = 0,
}: ForecastFanChartProps) {
  const { t, i18n } = useTranslation();
  const { data, error } = useForecastResult<ForecastViewResult>(
    analysisId,
    t("forecast.noAnalysis"),
  );

  if (error) return <div className="fcwf-companion-empty">{error}</div>;
  if (!data) {
    return <div className="fc-loading"><div className="fc-skeleton" /></div>;
  }

  const filtered = filterForecastSeriesData(
    data,
    data.input_data.series_ids?.[0] ?? "",
    [],
  );
  if (!filtered.predictions.length) {
    return (
      <div className="fcwf-companion-empty">
        {t("forecast.companion.insufficientData")}
      </div>
    );
  }

  return (
    <div className="fcwf-companion fcwf-companion-fan">
      {/* key remount: the canvas can lose its bitmap while the card body is
          clipped/composited during collapse, so expanding re-initializes the
          chart at full size. Full (non-compact) mode for the zoom controls. */}
      <ForecastChart
        key={resizeSignal}
        history={filtered.history.slice(-FAN_HISTORY_WINDOW)}
        predictions={filtered.predictions}
        scenarios={[]}
        variables={[]}
        annotations={[]}
        quantiles={{ q10: filtered.q10, q90: filtered.q90 }}
        frequency={data.frequency}
        endDate={data.input_summary.end}
        locale={i18n.language}
        targetColumn={data.target_column}
        fallbackName={data.name}
        labels={{
          history: t("forecast.view.historySeries"),
          forecast: t("forecast.view.forecastSeries"),
          confidence: t("forecast.view.confidenceRange"),
          forecastStart: t("forecast.chart.forecastStart"),
          annotationUser: t("forecast.notes.userSource"),
          annotationLlm: t("forecast.notes.llmSource"),
        }}
        layers={FAN_LAYERS}
        mode="main"
      />
    </div>
  );
}
