import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "../forecast-sections.css";
import "./forecast-scenarios.css";

interface ForecastScenario {
  id: string;
  name: string;
  description?: string | null;
  predictions: { value: number }[];
  params_modified?: { adjustment_percent?: number };
}

interface ForecastResult {
  scenarios: ForecastScenario[];
}

interface ForecastScenariosProps {
  analysisId: string;
  onAnalysisChanged: () => void;
}

export function ForecastScenarios({ analysisId, onAnalysisChanged }: ForecastScenariosProps) {
  const { t } = useTranslation();
  const [data, setData] = useState<ForecastResult | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [adjustment, setAdjustment] = useState("10");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void loadScenarioAnalysis(analysisId)
      .then((analysis) => {
        if (active) setData(analysis);
      })
      .catch(() => {
        if (active) setError(t("forecast.scenarios.loadFailed"));
      });
    return () => {
      active = false;
    };
  }, [analysisId, t]);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const updated = await invoke<ForecastResult>("create_forecast_scenario", {
        request: {
          analysis_id: analysisId,
          name,
          description,
          adjustment_percent: Number(adjustment),
        },
      });
      setData(updated);
      setName("");
      setDescription("");
      setAdjustment("10");
      onAnalysisChanged();
    } catch {
      setError(t("forecast.scenarios.saveFailed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">{t("forecast.nav.scenarios")}</span>
      </div>
      <div className="fcs-content">
        <form className="fcs-form" onSubmit={(event) => void handleSubmit(event)}>
          <input
            className="fcs-input"
            value={name}
            maxLength={80}
            onChange={(event) => setName(event.target.value)}
            placeholder={t("forecast.scenarios.name")}
          />
          <input
            className="fcs-input"
            value={description}
            maxLength={500}
            onChange={(event) => setDescription(event.target.value)}
            placeholder={t("forecast.scenarios.description")}
          />
          <div className="fcs-form-row">
            <input
              className="fcs-input fcs-input-number"
              type="number"
              min="-95"
              max="500"
              step="0.1"
              value={adjustment}
              onChange={(event) => setAdjustment(event.target.value)}
              aria-label={t("forecast.scenarios.adjustment")}
            />
            <button className="fcs-add-btn" type="submit" disabled={saving || !name.trim()}>
              {saving ? t("forecast.scenarios.saving") : t("forecast.scenarios.add")}
            </button>
          </div>
          {error && <p className="fcs-error">{error}</p>}
        </form>
        {data?.scenarios.length ? (
          <div className="fcs-list">
            {data.scenarios.map((scenario) => (
              <ScenarioRow key={scenario.id} scenario={scenario} />
            ))}
          </div>
        ) : (
          <div className="fcs-empty">
            <p className="fcs-empty-text">{t("forecast.scenarios.empty")}</p>
            <p className="fcs-empty-sub">{t("forecast.scenarios.emptySub")}</p>
          </div>
        )}
      </div>
    </div>
  );
}

function ScenarioRow({ scenario }: { scenario: ForecastScenario }) {
  const { t, i18n } = useTranslation();
  const adjustment = scenario.params_modified?.adjustment_percent;

  return (
    <div className="fcs-row">
      <div className="fcs-row-main">
        <span className="fcs-row-name">{scenario.name}</span>
        {scenario.description && <span className="fcs-row-description">{scenario.description}</span>}
      </div>
      <div className="fcs-row-meta">
        <span>{t("forecast.scenarios.points", { count: scenario.predictions.length })}</span>
        {typeof adjustment === "number" && (
          <span>{formatPercent(adjustment, i18n.language)}</span>
        )}
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

async function loadScenarioAnalysis(analysisId: string): Promise<ForecastResult> {
  return invoke<ForecastResult>("get_forecast_analysis", { id: analysisId });
}
