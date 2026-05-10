import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "../forecast-sections.css";
import "../forecast-history.css";

interface AnalysisMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  horizon: number;
  points: number;
  mape: number | null;
}

interface ForecastHistoryProps {
  onLoadAnalysis: (id: string) => void;
}

export function ForecastHistory({ onLoadAnalysis }: ForecastHistoryProps) {
  const { t } = useTranslation();
  const [analyses, setAnalyses] = useState<AnalysisMeta[]>([]);
  const [search, setSearch] = useState("");

  useEffect(() => {
    invoke<AnalysisMeta[]>("list_forecast_analyses")
      .then(setAnalyses)
      .catch(() => {});
  }, []);

  const filtered = search
    ? analyses.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()))
    : analyses;

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">{t("forecast.nav.history")}</span>
      </div>
      <div className="fcs-content">
        <div className="fch-search">
          <input
            className="fch-search-input"
            placeholder="Rechercher..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        {filtered.length === 0 ? (
          <div className="fcs-empty">
            <p className="fcs-empty-text">Aucune analyse sauvegardée.</p>
          </div>
        ) : (
          <div className="fch-list">
            {filtered.map((a) => (
              <button key={a.id} className="fch-card" onClick={() => onLoadAnalysis(a.id)}>
                <span className="fch-name">{a.name}</span>
                <span className="fch-meta">
                  {a.model} · {a.points} pts · H{a.horizon}
                  {a.mape != null && ` · MAPE ${a.mape.toFixed(1)}%`}
                </span>
                <span className="fch-date">{new Date(a.created_at).toLocaleDateString()}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
