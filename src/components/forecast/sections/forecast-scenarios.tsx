import "../forecast-sections.css";

interface ForecastScenariosProps {
  analysisId: string;
}

export function ForecastScenarios({ analysisId: _analysisId }: ForecastScenariosProps) {
  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">Scénarios what-if</span>
        <button className="fcs-add-btn">+ Nouveau scénario</button>
      </div>
      <div className="fcs-empty">
        <p className="fcs-empty-text">
          Aucun scénario créé pour cette analyse.
        </p>
        <p className="fcs-empty-sub">
          Créez des scénarios pour comparer différentes hypothèses
          et visualiser leur impact sur les prédictions.
        </p>
      </div>
    </div>
  );
}
