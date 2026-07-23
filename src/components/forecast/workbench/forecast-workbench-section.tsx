import { useTranslation } from "react-i18next";
import { ForecastEvaluationView } from "../evaluation/forecast-evaluation-view";
import { ForecastWorkbenchReport } from "./forecast-workbench-report";
import type {
  ForecastWorkbenchSection,
  ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";

interface ForecastWorkbenchSectionProps {
  section: ForecastWorkbenchSection;
  snapshot: ForecastWorkbenchSnapshot;
}

export function ForecastWorkbenchSectionContent({
  section,
  snapshot,
}: ForecastWorkbenchSectionProps) {
  const { t } = useTranslation();
  if (section === "report" && snapshot.context.analysis_id) {
    return <ForecastWorkbenchReport analysisId={snapshot.context.analysis_id} />;
  }
  if (section === "report") {
    return (
      <div className="fcw-foundation">
        <span>{t("forecast.workbench.evaluation.noAnalysis")}</span>
        <p>{t("forecast.workbench.evaluation.noAnalysisDescription")}</p>
      </div>
    );
  }
  if ((section === "evaluation" || section === "comparison") && snapshot.context.analysis_id) {
    return (
      <ForecastEvaluationView
        analysisId={snapshot.context.analysis_id}
        mode={section}
      />
    );
  }
  if (section === "evaluation" || section === "comparison") {
    return (
      <div className="fcw-foundation">
        <span>{t("forecast.workbench.evaluation.noAnalysis")}</span>
        <p>{t("forecast.workbench.evaluation.noAnalysisDescription")}</p>
      </div>
    );
  }
  return (
    <div className="fcw-foundation">
      <span>{t("forecast.workbench.foundationTitle")}</span>
      <p>{t("forecast.workbench.foundationDescription")}</p>
    </div>
  );
}
