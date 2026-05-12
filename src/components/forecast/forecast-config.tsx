import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastDraftData } from "./forecast-data";
import { ForecastConfigContext } from "./forecast-config-context";
import { buildForecastContextProfile } from "./forecast-context-profile";
import { buildLaunchErrorKey } from "./forecast-config-validation";
import { FieldSelect, OptionalFieldSelect } from "./forecast-config-fields";
import {
  isForecastModelSelectable,
  type ForecastModelEntry,
  type ForecastModelsResponse,
} from "./forecast-model-meta";
import { ForecastConfigModelPicker } from "./forecast-config-model-picker";
import "./forecast-config.css";

interface ForecastConfigProps {
  draft: ForecastDraftData;
  launching: boolean;
  error: string | null;
  onLaunch: (config: LaunchConfig) => void;
  onBack: () => void;
}

export interface LaunchConfig {
  targetColumn: string;
  dateColumn: string;
  seriesColumn: string | null;
  covariates: string[];
  horizon: number;
  frequency: string;
  model: string;
  confidence: number;
}

const FREQUENCIES = [
  "D", "W", "M", "Q", "Y", "H", "T",
];

export function ForecastConfig({ draft, launching, error, onLaunch, onBack }: ForecastConfigProps) {
  const { t } = useTranslation();
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [target, setTarget] = useState(draft.columns[1] ?? "");
  const [dateCol, setDateCol] = useState(draft.columns[0] ?? "");
  const [seriesCol, setSeriesCol] = useState("");
  const [covariates, setCovariates] = useState<string[]>([]);
  const [horizon, setHorizon] = useState(12);
  const [frequency, setFrequency] = useState("M");
  const [model, setModel] = useState("");
  const [confidence, setConfidence] = useState(0.9);

  useEffect(() => {
    invoke<ForecastModelsResponse>("list_forecast_models")
      .then((r) => {
        const visibleModels = r.models.filter(isForecastModelSelectable);
        setModels(visibleModels);
        const first = visibleModels[0];
        if (first) setModel(first.id);
      })
      .catch(() => {});
  }, []);

  const covariateOptions = draft.columns.filter(
    (column) => column !== target && column !== dateCol && column !== seriesCol
  );
  const seriesOptions = draft.columns.filter(
    (column) => column !== target && column !== dateCol
  );
  const selectedCovariates = covariates.filter(
    (column) => column !== target && column !== dateCol && column !== seriesCol
  );
  const selectedModel = useMemo(
    () => models.find((entry) => entry.id === model) ?? null,
    [models, model]
  );
  const contextProfile = useMemo(
    () =>
      buildForecastContextProfile(
        draft.rows,
        target,
        selectedCovariates,
        seriesCol || null
      ),
    [draft.rows, target, selectedCovariates, seriesCol]
  );

  const configError = buildLaunchErrorKey(selectedModel, contextProfile, horizon);

  const canLaunch = target.trim() !== "" && dateCol.trim() !== "" && model !== "" && horizon > 0 && configError === null;

  return (
    <div className="fcc-root">
      <div className="fcc-header">
        <button className="fcc-back" onClick={onBack}>{t("forecast.config.back")}</button>
        <span className="fcc-title">{t("forecast.config.title")}</span>
      </div>
      <div className="fcc-form">
        <div className="fcc-source">
          <span>{draft.sourceName}</span>
          <span>{t("forecast.config.rows", { count: draft.rowCount })}</span>
        </div>
        <ForecastConfigContext
          profile={contextProfile}
          horizon={horizon}
          model={selectedModel}
        />
        <FieldSelect label={t("forecast.config.target")} value={target} onChange={setTarget} options={draft.columns} />
        <FieldSelect label={t("forecast.config.dateColumn")} value={dateCol} onChange={setDateCol} options={draft.columns} />
        <OptionalFieldSelect
          label={t("forecast.config.seriesColumn")}
          value={seriesCol}
          onChange={setSeriesCol}
          emptyLabel={t("forecast.config.noSeriesColumn")}
          options={seriesOptions}
        />
        {covariateOptions.length > 0 && (
          <div className="fcc-field">
            <span className="fcc-label">{t("forecast.config.covariates")}</span>
            <div className="fcc-chips">
              {covariateOptions.map((column) => {
                const active = selectedCovariates.includes(column);
                return (
                  <button
                    key={column}
                    className={`fcc-chip ${active ? "fcc-chip-active" : ""}`}
                    type="button"
                    onClick={() => {
                      setCovariates((current) =>
                        current.includes(column)
                          ? current.filter((item) => item !== column)
                          : [...current, column]
                      );
                    }}
                  >
                    {column}
                  </button>
                );
              })}
            </div>
          </div>
        )}
        <div className="fcc-row">
          <div className="fcc-field fcc-half">
            <label className="fcc-label" htmlFor="fcc-horizon">{t("forecast.config.horizon")}</label>
            <input className="fcc-input" id="fcc-horizon" type="number" min={1} max={5000}
              value={horizon} onChange={(e) => setHorizon(Number(e.target.value))} />
          </div>
          <div className="fcc-field fcc-half">
            <label className="fcc-label" htmlFor="fcc-freq">{t("forecast.config.frequency")}</label>
            <select className="fcc-select" id="fcc-freq" value={frequency} onChange={(e) => setFrequency(e.target.value)}>
              {FREQUENCIES.map((f) => <option key={f} value={f}>{t(`forecast.frequency.${f}`)}</option>)}
            </select>
          </div>
        </div>
        <div className="fcc-field">
          <span className="fcc-label">{t("forecast.config.model")}</span>
          <ForecastConfigModelPicker
            models={models}
            selectedId={model}
            onSelect={setModel}
          />
        </div>
        <div className="fcc-field">
          <label className="fcc-label" htmlFor="fcc-confidence">{t("forecast.config.confidence")}: {Math.round(confidence * 100)}%</label>
          <input className="fcc-range" id="fcc-confidence" type="range" min={0.5} max={0.99} step={0.01}
            value={confidence} onChange={(e) => setConfidence(Number(e.target.value))} />
        </div>
        {(configError || error) && (
          <p className="fcc-error">
            {configError ? t(configError, { future: contextProfile.futureRows, horizon }) : error}
          </p>
        )}
      </div>
      <div className="fcc-footer">
        <button className="fcc-launch" disabled={!canLaunch || launching}
          onClick={() => onLaunch({
            targetColumn: target,
            dateColumn: dateCol,
            seriesColumn: seriesCol.trim() || null,
            covariates: selectedCovariates,
            horizon,
            frequency,
            model,
            confidence,
          })}>
          {launching ? t("forecast.config.launching") : t("forecast.config.launch")}
        </button>
      </div>
    </div>
  );
}
