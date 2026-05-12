import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Pencil, Trash } from "@/components/ui/icons";
import type { ForecastScenario } from "./forecast-scenario-types";

interface ForecastScenarioRowProps {
  scenario: ForecastScenario;
  onEdit: (scenario: ForecastScenario) => void;
  onDelete: (scenarioId: string) => void;
}

export function ForecastScenarioRow({
  scenario,
  onEdit,
  onDelete,
}: ForecastScenarioRowProps) {
  const { t, i18n } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const adjustment = scenario.params_modified?.adjustment_percent;
  const kind = scenario.params_modified?.kind;

  useEffect(() => {
    if (!confirmDelete) return;

    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setConfirmDelete(false);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setConfirmDelete(false);
      }
      if (event.key === "Enter") {
        event.preventDefault();
        onDelete(scenario.id);
        setConfirmDelete(false);
      }
    };

    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [confirmDelete, onDelete, scenario.id]);

  return (
    <div className="fcs-row">
      <div className="fcs-row-main">
        <span className="fcs-row-name">{scenario.name}</span>
        {scenario.description && <span className="fcs-row-description">{scenario.description}</span>}
      </div>
      <div className="fcs-row-meta">
        <span className="fcs-row-summary">
          {kind === "percent_adjustment" ? t("forecast.scenarios.percentType") : ""}
          {kind === "percent_adjustment" ? " " : ""}
          {t("forecast.scenarios.points", { count: scenario.predictions.length })}
          {typeof adjustment === "number" ? ` ${formatPercent(adjustment, i18n.language)}` : ""}
        </span>
        <div ref={rootRef} className="fcs-row-actions">
          {confirmDelete && (
            <button
              className="fcs-confirm-delete"
              type="button"
              onClick={() => {
                onDelete(scenario.id);
                setConfirmDelete(false);
              }}
            >
              {t("forecast.scenarios.confirmDelete")}
            </button>
          )}
          <button
            className="fcs-icon-btn"
            type="button"
            onClick={() => onEdit(scenario)}
            title={t("forecast.scenarios.edit")}
          >
            <Pencil size={13} />
          </button>
          <button
            className="fcs-icon-btn fcs-icon-btn-danger"
            type="button"
            onClick={() => setConfirmDelete(true)}
            title={t("forecast.scenarios.delete")}
          >
            <Trash size={13} />
          </button>
        </div>
      </div>
    </div>
  );
}

function formatPercent(value: number, locale: string): string {
  return new Intl.NumberFormat(locale, {
    signDisplay: "always",
    maximumFractionDigits: 1,
  }).format(value) + "%";
}
