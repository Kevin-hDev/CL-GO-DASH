import { useTranslation } from "react-i18next";
import "../forecast-sections.css";

interface ForecastAnalysisProps {
  analysisId: string;
}

export function ForecastAnalysis({ analysisId: _analysisId }: ForecastAnalysisProps) {
  const { t } = useTranslation();

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">{t("forecast.nav.analysis")}</span>
      </div>
      <div className="fcs-empty">
        <p className="fcs-empty-text">{t("forecast.nav.analysis")} — coming soon</p>
      </div>
    </div>
  );
}
