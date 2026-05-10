import { useTranslation } from "react-i18next";
import "../forecast-sections.css";

interface ForecastScenariosProps {
  analysisId: string;
}

export function ForecastScenarios({ analysisId: _analysisId }: ForecastScenariosProps) {
  const { t } = useTranslation();

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">{t("forecast.nav.scenarios")}</span>
      </div>
      <div className="fcs-empty">
        <p className="fcs-empty-text">{t("forecast.nav.scenarios")} — coming soon</p>
      </div>
    </div>
  );
}
