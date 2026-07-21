import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ForecastWorkbenchModelControl } from "./forecast-workbench-model-control";
import { ForecastWorkbenchNav } from "./forecast-workbench-nav";
import type { ForecastWorkbenchSection } from "./forecast-workbench-types";
import { useForecastWorkbenchContext } from "./use-forecast-workbench-context";
import "./forecast-workbench.css";

export function ForecastWorkbenchWindow() {
  const { t } = useTranslation();
  const [section, setSection] = useState<ForecastWorkbenchSection>("data");
  const { snapshot, loading, failed } = useForecastWorkbenchContext();

  if (loading) {
    return <div className="fcw-state">{t("forecast.workbench.loading")}</div>;
  }
  if (failed || !snapshot) {
    return <div className="fcw-state fcw-state-error">{t("forecast.workbench.unavailable")}</div>;
  }
  return (
    <main className="fcw-shell">
      <header className="fcw-header">
        <div className="fcw-heading">
          <span className="fcw-kicker">{t("forecast.workbench.kicker")}</span>
          <h1>{snapshot.analysis_name ?? t("forecast.workbench.newAnalysis")}</h1>
          <span className="fcw-session">{snapshot.session_name}</span>
        </div>
        <ForecastWorkbenchModelControl />
      </header>
      <div className="fcw-layout">
        <ForecastWorkbenchNav active={section} onChange={setSection} />
        <section className="fcw-content" aria-labelledby="fcw-section-title">
          <div className="fcw-content-heading">
            <span className="fcw-step">{t("forecast.workbench.workspace")}</span>
            <h2 id="fcw-section-title">{t(`forecast.workbench.sections.${section}`)}</h2>
            <p>{t(`forecast.workbench.sectionDescriptions.${section}`)}</p>
          </div>
          <div className="fcw-foundation">
            <span>{t("forecast.workbench.foundationTitle")}</span>
            <p>{t("forecast.workbench.foundationDescription")}</p>
          </div>
        </section>
      </div>
    </main>
  );
}
