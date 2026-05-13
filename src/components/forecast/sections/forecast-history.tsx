import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ForecastHistoryRow } from "./forecast-history-row";
import "../forecast-sections.css";
import "../forecast-history.css";

export interface AnalysisMeta {
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
  const { t } = useTranslation();
  const [analyses, setAnalyses] = useState<AnalysisMeta[]>([]);
  const [search, setSearch] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<AnalysisMeta[]>("list_forecast_analyses")
      .then(setAnalyses)
      .catch(() => {});
  }, []);

  const filtered = search
    ? analyses.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()))
    : analyses;

  const handleRename = async (id: string, name: string) => {
    try {
      const renamed = await invoke<AnalysisMeta>("rename_forecast_analysis", { id, name });
      setAnalyses((items) => items.map((item) => (item.id === id ? renamed : item)));
      setError(null);
    } catch {
      setError(t("forecast.history.renameFailed"));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke("delete_forecast_analysis", { id });
      setAnalyses((items) => items.filter((item) => item.id !== id));
      setError(null);
    } catch {
      setError(t("forecast.history.deleteFailed"));
    }
  };

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
        {error && <p className="fch-error">{error}</p>}
        {filtered.length === 0 ? (
          <div className="fcs-empty">
            <p className="fcs-empty-text">{t("forecast.history.empty")}</p>
          </div>
        ) : (
          <div className="fch-list">
            {filtered.map((a) => (
              <ForecastHistoryRow
                key={a.id}
                analysis={a}
                onLoad={onLoadAnalysis}
                onRename={handleRename}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
