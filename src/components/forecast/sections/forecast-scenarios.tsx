import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "../forecast-sections.css";
import "./forecast-scenarios.css";
import { ForecastScenarioRow } from "./forecast-scenario-row";
import type { ForecastScenario, ForecastScenarioAnalysis } from "./forecast-scenario-types";
import { ForecastScenarioPreview } from "./forecast-scenario-preview";
import { ForecastScenarioForm } from "./forecast-scenario-form";
import { ForecastScenarioPicker } from "./forecast-scenario-picker";
import { useForecastChartResize } from "../use-forecast-chart-resize";

interface ForecastScenariosProps {
  analysisId: string;
  pickerOpen: boolean;
  onFocusAnalysis: (id: string) => void;
  onAnalysisChanged: () => void;
}

export function ForecastScenarios({
  analysisId,
  pickerOpen,
  onFocusAnalysis,
  onAnalysisChanged,
}: ForecastScenariosProps) {
  const { t } = useTranslation();
  const [data, setData] = useState<ForecastScenarioAnalysis | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [adjustment, setAdjustment] = useState("10");
  const [editingScenarioId, setEditingScenarioId] = useState<string | null>(null);
  const [selectedSeries, setSelectedSeries] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const chart = useForecastChartResize();

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

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const request = {
        analysis_id: analysisId,
        name,
        description,
        adjustment_percent: Number(adjustment),
      };
      const updated = editingScenarioId
        ? await invoke<ForecastScenarioAnalysis>("update_forecast_scenario", {
            request: { ...request, scenario_id: editingScenarioId },
          })
        : await invoke<ForecastScenarioAnalysis>("create_forecast_scenario", { request });
      setData(updated);
      resetForm();
      onAnalysisChanged();
    } catch {
      setError(
        editingScenarioId
          ? t("forecast.scenarios.updateFailed")
          : t("forecast.scenarios.saveFailed")
      );
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (scenarioId: string) => {
    setSaving(true);
    setError(null);
    try {
      const updated = await invoke<ForecastScenarioAnalysis>("delete_forecast_scenario", {
        analysisId,
        scenarioId,
      });
      setData(updated);
      if (editingScenarioId === scenarioId) resetForm();
      onAnalysisChanged();
    } catch {
      setError(t("forecast.scenarios.deleteFailed"));
    } finally {
      setSaving(false);
    }
  };

  const handleEdit = (scenario: ForecastScenario) => {
    setEditingScenarioId(scenario.id);
    setName(scenario.name);
    setDescription(scenario.description ?? "");
    setAdjustment(String(scenario.params_modified?.adjustment_percent ?? 0));
    setError(null);
  };

  return (
    <div className="fcs-root">
      <div className={`fcs-shell ${pickerOpen ? "is-picker-open" : ""}`}>
      <div className="fcs-content fcs-main">
        {data && (
          <div className="fcs-preview-shell">
            {data.input_data.series_ids && data.input_data.series_ids.length > 1 && (
              <div className="fcs-series-bar">
                <label className="fcs-series-label" htmlFor="fcs-series-select">
                  {t("forecast.view.series")}
                </label>
                <select
                  id="fcs-series-select"
                  className="fcs-series-select"
                  value={selectedSeries || data.input_data.series_ids[0]}
                  onChange={(event) => setSelectedSeries(event.target.value)}
                >
                  {data.input_data.series_ids.map((seriesId) => (
                    <option key={seriesId} value={seriesId}>
                      {seriesId}
                    </option>
                  ))}
                </select>
              </div>
            )}
            <ForecastScenarioPreview
              analysis={data}
              scenarioName={name}
              adjustmentPercent={Number(adjustment) || 0}
              selectedSeries={selectedSeries}
              chartHeight={chart.chartHeight}
              isResizing={chart.isResizing}
            />
            <div
              className="fcs-chart-resize"
              onPointerDown={chart.startResize}
              onDoubleClick={chart.resetHeight}
            />
          </div>
        )}
        <ForecastScenarioForm
          name={name}
          description={description}
          adjustment={adjustment}
          editing={Boolean(editingScenarioId)}
          saving={saving}
          error={error}
          onNameChange={setName}
          onDescriptionChange={setDescription}
          onAdjustmentChange={setAdjustment}
          onSubmit={(event) => void handleSubmit(event)}
          onCancel={resetForm}
        />
        {data?.scenarios.length ? (
          <div className="fcs-list">
            {data.scenarios.map((scenario) => (
              <ForecastScenarioRow
                key={scenario.id}
                scenario={scenario}
                onEdit={handleEdit}
                onDelete={(scenarioId) => void handleDelete(scenarioId)}
              />
            ))}
          </div>
        ) : (
          <div className="fcs-empty">
            <p className="fcs-empty-text">{t("forecast.scenarios.empty")}</p>
            <p className="fcs-empty-sub">{t("forecast.scenarios.emptySub")}</p>
          </div>
        )}
      </div>
      <ForecastScenarioPicker
        open={pickerOpen}
        currentAnalysisId={analysisId}
        onSelectAnalysis={onFocusAnalysis}
      />
      </div>
    </div>
  );

  function resetForm() {
    setEditingScenarioId(null);
    setName("");
    setDescription("");
    setAdjustment("10");
  }
}

async function loadScenarioAnalysis(analysisId: string): Promise<ForecastScenarioAnalysis> {
  return invoke<ForecastScenarioAnalysis>("get_forecast_analysis", { id: analysisId });
}
