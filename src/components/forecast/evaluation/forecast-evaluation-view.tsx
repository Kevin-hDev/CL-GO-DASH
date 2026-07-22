import { useTranslation } from "react-i18next";
import { ForecastEvaluationTable } from "./forecast-evaluation-table";
import { useForecastEvaluation } from "./use-forecast-evaluation";
import "./forecast-evaluation.css";

interface ForecastEvaluationViewProps {
  analysisId: string;
  mode: "evaluation" | "comparison";
}

export function ForecastEvaluationView({ analysisId, mode }: ForecastEvaluationViewProps) {
  const { t } = useTranslation();
  const { analysis, loading, running, loadFailed, runFailed, run } =
    useForecastEvaluation(analysisId);
  const evaluation = analysis?.evaluation;

  if (loading) {
    return <div className="fcwe-state">{t("forecast.workbench.evaluation.loading")}</div>;
  }
  if (loadFailed || !analysis) {
    return <div className="fcwe-state fcwe-state-error">{t("forecast.workbench.evaluation.loadFailed")}</div>;
  }
  return (
    <div className="fcwe-root">
      {runFailed ? (
        <p className="fcwe-error" role="alert">
          {t("forecast.workbench.evaluation.runFailed")}
        </p>
      ) : null}
      <div className="fcwe-toolbar">
        <div className="fcwe-summary">
          {evaluation ? (
            <>
              <strong>{t("forecast.workbench.evaluation.rollingBacktest")}</strong>
              <span>
                {t("forecast.workbench.evaluation.summary", {
                  windows: evaluation.windows,
                  horizon: evaluation.horizon,
                })}
              </span>
            </>
          ) : (
            <strong>{t("forecast.workbench.evaluation.notRun")}</strong>
          )}
        </div>
        {mode === "evaluation" ? (
          <button className="fcwe-run" type="button" disabled={running} onClick={() => void run()}>
            {running
              ? t("forecast.workbench.evaluation.running")
              : t("forecast.workbench.evaluation.run")}
          </button>
        ) : null}
      </div>
      {evaluation?.warning ? (
        <p className="fcwe-warning">
          {t(`forecast.workbench.evaluation.planWarnings.${evaluation.warning}`, {
            defaultValue: t("forecast.workbench.evaluation.planWarnings.unavailable"),
          })}
        </p>
      ) : null}
      {evaluation?.results.length ? (
        <ForecastEvaluationTable
          results={evaluation.results}
          currentModel={analysis.model}
          t={t}
        />
      ) : (
        <div className="fcwe-empty">
          <strong>{t("forecast.workbench.evaluation.emptyTitle")}</strong>
          <p>
            {t(
              mode === "evaluation"
                ? "forecast.workbench.evaluation.emptyEvaluation"
                : "forecast.workbench.evaluation.emptyComparison",
            )}
          </p>
        </div>
      )}
    </div>
  );
}
