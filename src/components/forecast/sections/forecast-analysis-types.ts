export interface ForecastAnalysisPoint {
  date: string;
  value: number;
  series_id?: string | null;
}

export interface ForecastAnalysisData {
  id: string;
  name: string;
  target_column?: string;
  series_column?: string | null;
  model: string;
  horizon: number;
  frequency: string;
  input_summary: { end: string };
  input_data: {
    rows?: Record<string, unknown>[];
    series_ids?: string[];
    history: ForecastAnalysisPoint[];
  };
  covariates_used?: string[];
  predictions: ForecastAnalysisPoint[];
  quantiles: { q10: number[]; q90: number[] };
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null } | null;
}

export interface AnalysisCard {
  labelKey: string;
  value: string;
  tone?: "neutral" | "warn" | "good";
}

export interface AnalysisEvent {
  id: string;
  label: string;
  value: string;
  meta: string;
  severity?: "low" | "medium" | "high";
}

export interface VariableInsight {
  name: string;
  score: number;
  detail: string;
}
