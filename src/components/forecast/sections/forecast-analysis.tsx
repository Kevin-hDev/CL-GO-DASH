import "../forecast-sections.css";

interface ForecastAnalysisProps {
  analysisId: string;
}

export function ForecastAnalysis({ analysisId: _analysisId }: ForecastAnalysisProps) {
  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">Analyse</span>
      </div>
      <div className="fcs-content">
        <div className="fcs-accordion">
          <details className="fcs-details">
            <summary className="fcs-summary">Décomposition</summary>
            <div className="fcs-details-body">
              <p className="fcs-empty-text">Trend, saisonnalité et résidus — disponible après calcul.</p>
            </div>
          </details>
          <details className="fcs-details">
            <summary className="fcs-summary">Feature Importance</summary>
            <div className="fcs-details-body">
              <p className="fcs-empty-text">Importance des covariables — disponible avec covariables.</p>
            </div>
          </details>
          <details className="fcs-details">
            <summary className="fcs-summary">Anomalies</summary>
            <div className="fcs-details-body">
              <p className="fcs-empty-text">Détection automatique des anomalies dans les données.</p>
            </div>
          </details>
        </div>
      </div>
    </div>
  );
}
