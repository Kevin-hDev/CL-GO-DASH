import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./forecast-empty.css";

interface ForecastAnalysisMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  horizon: number;
  mape: number | null;
}

interface ForecastEmptyProps {
  onLoadAnalysis: (id: string) => void;
}

export function ForecastEmpty({ onLoadAnalysis }: ForecastEmptyProps) {
  const [recent, setRecent] = useState<ForecastAnalysisMeta[]>([]);

  useEffect(() => {
    invoke<ForecastAnalysisMeta[]>("list_forecast_analyses")
      .then(setRecent)
      .catch(() => setRecent([]));
  }, []);

  return (
    <div className="fc-empty">
      <div className="fc-empty-icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="var(--ink-faint)"
          strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M6 36L14 24L22 30L32 16L42 22" />
          <path d="M32 16L42 16L42 22" opacity="0.5" />
          <line x1="6" y1="42" x2="42" y2="42" opacity="0.3" />
        </svg>
      </div>
      <p className="fc-empty-title">Aucune analyse en cours</p>
      <p className="fc-empty-sub">
        Demandez à l&apos;agent de lancer un forecast, ou importez des données.
      </p>
      <div className="fc-empty-actions">
        <button className="fc-empty-btn fc-empty-btn-primary">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor"
            strokeWidth="1.5" strokeLinecap="round">
            <circle cx="7" cy="7" r="5.5" />
            <path d="M7 4.5v5M4.5 7h5" />
          </svg>
          Importer (CSV, Excel)
        </button>
        <button className="fc-empty-btn">Coller des données</button>
        <button className="fc-empty-btn">Depuis une URL</button>
      </div>
      {recent.length > 0 && (
        <div className="fc-recent">
          <p className="fc-recent-title">Analyses récentes</p>
          {recent.slice(0, 5).map((a) => (
            <button key={a.id} className="fc-recent-item" onClick={() => onLoadAnalysis(a.id)}>
              <span className="fc-recent-name">{a.name}</span>
              <span className="fc-recent-meta">
                {a.model} · H{a.horizon}
                {a.mape != null && ` · ${a.mape.toFixed(1)}%`}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
