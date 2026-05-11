import { buildPeriodMeta } from "./forecast-view-format";

export function PeriodCell({
  index,
  rawDate,
  endDate,
  frequency,
  locale,
}: {
  index: number;
  rawDate: string;
  endDate: string;
  frequency: string;
  locale: string;
}) {
  const period = buildPeriodMeta(index, rawDate, endDate, frequency, locale);

  return (
    <div className="fc-table-period">
      <span className="fc-table-step">{period.stepLabel}</span>
      <span className="fc-table-date">{period.secondaryLabel}</span>
    </div>
  );
}

export function ValueCell({
  unitLabel,
  formattedValue,
}: {
  unitLabel: string | null;
  formattedValue: string;
}) {
  const inlineUnit = unitLabel && unitLabel !== "€" ? unitLabel : null;

  return (
    <div className="fc-table-value-block">
      <span className="fc-table-value">{formattedValue}</span>
      {inlineUnit && <span className="fc-table-unit">{inlineUnit}</span>}
    </div>
  );
}

export function KpiRow({ metrics }: {
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null };
}) {
  const kpis = [
    { label: "MAPE", value: metrics.mape, suffix: "%" },
    { label: "MAE", value: metrics.mae, suffix: "" },
    { label: "CRPS", value: metrics.crps, suffix: "" },
    { label: "Bias", value: metrics.bias, suffix: "" },
  ];

  return (
    <div className="fc-kpi-row">
      {kpis.map((kpi) => (
        <div key={kpi.label} className="fc-kpi-card">
          <span className="fc-kpi-label">{kpi.label}</span>
          <span className="fc-kpi-value">
            {kpi.value != null ? `${kpi.value.toFixed(2)}${kpi.suffix}` : "—"}
          </span>
        </div>
      ))}
    </div>
  );
}

export function ChartPreview({
  history,
  predictions,
}: {
  history: { date: string; value: number }[];
  predictions: { date: string; value: number }[];
}) {
  const combined = [...history, ...predictions];
  if (combined.length < 2) return null;

  const values = combined.map((point) => point.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const width = 400;
  const height = 120;
  const padding = 10;

  const toPoint = (value: number, index: number, total: number) => {
    const x = (index / (total - 1)) * (width - padding * 2) + padding;
    const y = height - padding - ((value - min) / range) * (height - padding * 2);
    return `${x},${y}`;
  };

  const historyPoints = history.map((point, index) => {
    return toPoint(point.value, index, combined.length);
  }).join(" ");
  const predictionPoints = predictions.map((point, index) => {
    return toPoint(point.value, history.length + index, combined.length);
  }).join(" ");
  const separatorIndex = history.length - 1;
  const separatorX = separatorIndex > 0
    ? ((separatorIndex / (combined.length - 1)) * (width - padding * 2) + padding)
    : null;

  return (
    <>
      {historyPoints && (
        <polyline
          points={historyPoints}
          fill="none"
          stroke="var(--fc-line-history)"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
      {predictionPoints && (
        <polyline
          points={predictionPoints}
          fill="none"
          stroke="var(--fc-line-predict)"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
      {separatorX != null && (
        <line
          x1={separatorX}
          x2={separatorX}
          y1={padding}
          y2={height - padding}
          stroke="var(--fc-separator)"
          strokeDasharray="4 4"
          strokeWidth="1"
        />
      )}
    </>
  );
}
