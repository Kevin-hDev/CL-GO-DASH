import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { formatForecastValue, inferMetricMeta } from "../forecast-view-format";
import { ChartPreview, KpiRow, PeriodCell, ValueCell } from "../forecast-view-widgets";
import "../forecast-view.css";

interface ForecastResult {
  id: string;
  name: string;
  target_column?: string;
  model: string;
  horizon: number;
  frequency: string;
  input_summary: {
    end: string;
  };
  input_data: {
    history: { date: string; value: number }[];
  };
  predictions: { date: string; value: number }[];
  quantiles: { q10: number[]; q50: number[]; q90: number[] };
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null } | null;
}

interface ForecastViewProps {
  analysisId: string;
}

export function ForecastView({ analysisId }: ForecastViewProps) {
  const { t, i18n } = useTranslation();
  const [data, setData] = useState<ForecastResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<ForecastResult>("get_forecast_analysis", { id: analysisId })
      .then(setData)
      .catch(() => setError(t("forecast.noAnalysis")));
  }, [analysisId, t]);

  if (error) return <div className="fc-error">{error}</div>;
  if (!data) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  const metric = inferMetricMeta(i18n.language, data.target_column, data.name);

  return (
    <div className="fc-view">
      {data.metrics && <KpiRow metrics={data.metrics} />}
      <div className="fc-chart-area">
        <div className="fc-chart-placeholder">
          <svg width="100%" height="120" viewBox="0 0 400 120" preserveAspectRatio="none">
            <ChartPreview history={data.input_data.history} predictions={data.predictions} />
          </svg>
        </div>
      </div>
      <div className="fc-predictions-table">
        <div className="fc-table-head">
          <span>{t("forecast.view.period")}</span>
          <span>{metric.columnTitle}</span>
        </div>
        <div className="fc-table-body">
          {data.predictions.slice(0, 20).map((p, i) => (
            <div key={i} className="fc-table-row">
              <PeriodCell
                index={i}
                rawDate={p.date}
                endDate={data.input_summary.end}
                frequency={data.frequency}
                locale={i18n.language}
              />
              <ValueCell
                unitLabel={metric.unitLabel}
                formattedValue={formatForecastValue(p.value, i18n.language, metric)}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
