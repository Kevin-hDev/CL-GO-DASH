import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "../forecast-view.css";

interface ForecastResult {
  id: string;
  name: string;
  model: string;
  horizon: number;
  frequency: string;
  predictions: { date: string; value: number }[];
  quantiles: { q10: number[]; q50: number[]; q90: number[] };
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null } | null;
}

interface ForecastViewProps {
  analysisId: string;
}

export function ForecastView({ analysisId }: ForecastViewProps) {
  const [data, setData] = useState<ForecastResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<ForecastResult>("get_forecast_analysis", { id: analysisId })
      .then(setData)
      .catch((e: unknown) => setError(String(e)));
  }, [analysisId]);

  if (error) return <div className="fc-error">{error}</div>;
  if (!data) return <div className="fc-loading"><div className="fc-skeleton" /></div>;

  return (
    <div className="fc-view">
      {data.metrics && <KpiRow metrics={data.metrics} />}
      <div className="fc-chart-area">
        <div className="fc-chart-placeholder">
          <svg width="100%" height="120" viewBox="0 0 400 120" preserveAspectRatio="none">
            <ChartPreview predictions={data.predictions} />
          </svg>
        </div>
      </div>
      <div className="fc-predictions-table">
        <div className="fc-table-head">
          <span>Date</span>
          <span>Prédiction</span>
        </div>
        <div className="fc-table-body">
          {data.predictions.slice(0, 20).map((p, i) => (
            <div key={i} className="fc-table-row">
              <span className="fc-table-date">{p.date}</span>
              <span className="fc-table-value">{p.value.toFixed(2)}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function KpiRow({ metrics }: { metrics: NonNullable<ForecastResult["metrics"]> }) {
  const kpis = [
    { label: "MAPE", value: metrics.mape, suffix: "%" },
    { label: "MAE", value: metrics.mae, suffix: "" },
    { label: "CRPS", value: metrics.crps, suffix: "" },
    { label: "Biais", value: metrics.bias, suffix: "" },
  ];
  return (
    <div className="fc-kpi-row">
      {kpis.map((k) => (
        <div key={k.label} className="fc-kpi-card">
          <span className="fc-kpi-label">{k.label}</span>
          <span className="fc-kpi-value">
            {k.value != null ? `${k.value.toFixed(2)}${k.suffix}` : "—"}
          </span>
        </div>
      ))}
    </div>
  );
}

function ChartPreview({ predictions }: { predictions: ForecastResult["predictions"] }) {
  if (predictions.length === 0) return null;
  const values = predictions.map((p) => p.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const w = 400;
  const h = 120;
  const pad = 10;

  const points = values.map((v, i) => {
    const x = (i / (values.length - 1)) * (w - pad * 2) + pad;
    const y = h - pad - ((v - min) / range) * (h - pad * 2);
    return `${x},${y}`;
  }).join(" ");

  return (
    <polyline
      points={points}
      fill="none"
      stroke="var(--fc-line-predict)"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  );
}
