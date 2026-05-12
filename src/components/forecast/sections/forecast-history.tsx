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
  scenarios_count: number;
}

interface ForecastHistoryProps {
  onLoadAnalysis: (id: string) => void;
}

export function ForecastHistory({ onLoadAnalysis }: ForecastHistoryProps) {
  const { t, i18n } = useTranslation();
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
            placeholder={t("forecast.history.searchPlaceholder")}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        {filtered.length === 0 ? (
          <div className="fcs-empty">
            <p className="fcs-empty-text">{t("forecast.history.empty")}</p>
          </div>
        ) : (
          <div className="fch-list">
            {filtered.map((a) => (
              <button key={a.id} className="fch-card" onClick={() => onLoadAnalysis(a.id)}>
                <span className="fch-name-row">
                  <span className="fch-name">{a.name}</span>
                  {a.scenarios_count > 0 && <span className="fch-scenario-dot" />}
                </span>
                <span className="fch-meta">
                  {a.model} · {t("forecast.history.points", { count: a.points })} · {t("forecast.history.horizonShort", { count: a.horizon })}
                  {a.mape != null && ` · ${t("forecast.history.mapeShort", { value: a.mape.toFixed(1) })}`}
                </span>
                <span className="fch-date">{formatTimestamp(a.created_at, i18n.language)}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function formatTimestamp(value: string, language: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(language, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}
