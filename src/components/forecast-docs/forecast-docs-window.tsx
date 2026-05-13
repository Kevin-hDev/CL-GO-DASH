import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { FORECAST_DOC_PAGES } from "./forecast-docs-data";
import { ForecastDocsAccordion } from "./forecast-docs-accordion";
import { ForecastDocsMarkdown } from "./forecast-docs-markdown";
import { ForecastDocsSidebar } from "./forecast-docs-sidebar";
import "./forecast-docs.css";

export function ForecastDocsWindow() {
  const { t } = useTranslation();
  const [activeId, setActiveId] = useState(FORECAST_DOC_PAGES[0]?.id ?? "");
  const activePage = useMemo(
    () => FORECAST_DOC_PAGES.find((page) => page.id === activeId) ?? FORECAST_DOC_PAGES[0],
    [activeId],
  );

  return (
    <main className="fd-window">
      <ForecastDocsSidebar
        pages={FORECAST_DOC_PAGES}
        activeId={activePage.id}
        onSelect={setActiveId}
      />
      <article className="fd-content">
        <header className="fd-header">
          <div>
            <div className="fd-kicker">{t("forecast.docs.kicker")}</div>
            <h1>{activePage.title}</h1>
          </div>
          <span className="fd-status">{t("forecast.docs.status")}</span>
        </header>

        {activePage.summary && (
          <section className="fd-summary">
            <ForecastDocsMarkdown content={activePage.summary} />
          </section>
        )}

        <div className="fd-sections">
          {activePage.sections.map((section, index) => (
            <ForecastDocsAccordion
              key={section.id}
              section={section}
              defaultOpen={index === 0}
            />
          ))}
        </div>
      </article>
    </main>
  );
}
