import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "../forecast-sections.css";

interface Annotation {
  id: string;
  date: string;
  text: string;
  source: "user" | "llm";
}

interface ForecastResult {
  annotations: Annotation[];
}

interface ForecastNotesProps {
  analysisId: string;
}

export function ForecastNotes({ analysisId }: ForecastNotesProps) {
  const [annotations, setAnnotations] = useState<Annotation[]>([]);

  useEffect(() => {
    invoke<ForecastResult>("get_forecast_analysis", { id: analysisId })
      .then((r) => setAnnotations(r.annotations))
      .catch(() => {});
  }, [analysisId]);

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">Notes & Annotations</span>
      </div>
      <div className="fcs-content">
        {annotations.length === 0 ? (
          <div className="fcs-empty">
            <p className="fcs-empty-text">Aucune annotation pour le moment.</p>
            <p className="fcs-empty-sub">
              L&apos;agent peut ajouter des annotations automatiquement,
              ou vous pouvez en créer manuellement.
            </p>
          </div>
        ) : (
          <div className="fcn-timeline">
            {annotations.map((a) => (
              <div key={a.id} className="fcn-item">
                <span className="fcn-badge">
                  {a.source === "user" ? "📌" : "🤖"}
                </span>
                <div className="fcn-body">
                  <span className="fcn-date">{a.date}</span>
                  <span className="fcn-text">{a.text}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
