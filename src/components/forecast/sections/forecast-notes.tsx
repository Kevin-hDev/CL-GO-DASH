import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "../forecast-sections.css";

interface ForecastResult {
  annotations: { id: string; date: string; text: string; source: "user" | "llm" }[];
}

interface ForecastNotesProps {
  analysisId: string;
}

export function ForecastNotes({ analysisId }: ForecastNotesProps) {
  const { t } = useTranslation();
  const [annotations, setAnnotations] = useState<ForecastResult["annotations"]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    invoke<ForecastResult>("get_forecast_analysis", { id: analysisId })
      .then((result) => {
        if (!active) return;
        setAnnotations(result.annotations);
        setError(null);
      })
      .catch(() => {
        if (active) setError(t("forecast.notes.loadFailed"));
      });
    return () => {
      active = false;
    };
  }, [analysisId, t]);

  if (error) return <div className="fc-error">{error}</div>;

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">{t("forecast.nav.notes")}</span>
      </div>
      <div className="fcs-content">
        <div className="fcs-empty">
          <p className="fcs-empty-text">{t("forecast.notes.empty")}</p>
          <p className="fcs-empty-sub">
            {annotations.length > 0
              ? t("forecast.notes.designPendingWithCount", { count: annotations.length })
              : t("forecast.notes.designPending")}
          </p>
        </div>
      </div>
    </div>
  );
}
