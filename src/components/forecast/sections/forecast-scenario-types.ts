export interface ForecastScenario {
  id: string;
  name: string;
  description?: string | null;
  predictions: { date: string; value: number; series_id?: string | null }[];
  params_modified?: { kind?: string; adjustment_percent?: number };
}

export interface ForecastScenarioAnalysis {
  target_column?: string;
  frequency: string;
  input_summary: {
    end: string;
  };
  input_data: {
    series_ids?: string[];
    history: { date: string; value: number; series_id?: string | null }[];
  };
  predictions: { date: string; value: number; series_id?: string | null }[];
  quantiles: { q10: number[]; q90: number[] };
  scenarios: ForecastScenario[];
}
