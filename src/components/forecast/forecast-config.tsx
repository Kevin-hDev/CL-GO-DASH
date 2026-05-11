import { useId, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastDraftData } from "./forecast-data";
import "./forecast-config.css";

interface ModelEntry {
  id: string;
  display_name: string;
  provider_id: string;
  is_cloud: boolean;
  installed: boolean;
}

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
  const [models, setModels] = useState<ModelEntry[]>([]);
  const [target, setTarget] = useState(draft.columns[1] ?? "");
  const [dateCol, setDateCol] = useState(draft.columns[0] ?? "");
  const [horizon, setHorizon] = useState(12);
  const [frequency, setFrequency] = useState("M");
  const [model, setModel] = useState("");
  const [confidence, setConfidence] = useState(0.9);

  useEffect(() => {
    invoke<{ models: ModelEntry[] }>("list_forecast_models")
      .then((r) => {
        setModels(r.models);
        const first = r.models.find((m) => m.installed || m.is_cloud);
        if (first) setModel(first.id);
      })
      .catch(() => {});
  }, []);

  const localModels = models.filter((m) => !m.is_cloud);
  const cloudModels = models.filter((m) => m.is_cloud);

  const canLaunch = target.trim() !== "" && dateCol.trim() !== "" && model !== "" && horizon > 0;

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
        <FieldSelect label={t("forecast.config.target")} value={target} onChange={setTarget} options={draft.columns} />
        <FieldSelect label={t("forecast.config.dateColumn")} value={dateCol} onChange={setDateCol} options={draft.columns} />
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
          <label className="fcc-label" htmlFor="fcc-model">{t("forecast.config.model")}</label>
          <select className="fcc-select" id="fcc-model" value={model} onChange={(e) => setModel(e.target.value)}>
            <optgroup label={t("forecast.models.local")}>
              {localModels.map((m) => (
                <option key={m.id} value={m.id} disabled={!m.installed}>
                  {m.display_name} {m.installed ? "" : t("forecast.models.notInstalledSuffix")}
                </option>
              ))}
            </optgroup>
            <optgroup label={t("forecast.models.cloud")}>
              {cloudModels.map((m) => (
                <option key={m.id} value={m.id}>{m.display_name}</option>
              ))}
            </optgroup>
          </select>
        </div>
        <div className="fcc-field">
          <label className="fcc-label" htmlFor="fcc-confidence">{t("forecast.config.confidence")}: {Math.round(confidence * 100)}%</label>
          <input className="fcc-range" id="fcc-confidence" type="range" min={0.5} max={0.99} step={0.01}
            value={confidence} onChange={(e) => setConfidence(Number(e.target.value))} />
        </div>
        {error && <p className="fcc-error">{error}</p>}
      </div>
      <div className="fcc-footer">
        <button className="fcc-launch" disabled={!canLaunch || launching}
          onClick={() => onLaunch({ targetColumn: target, dateColumn: dateCol, horizon, frequency, model, confidence })}>
          {launching ? t("forecast.config.launching") : t("forecast.config.launch")}
        </button>
      </div>
    </div>
  );
}

function FieldSelect({ label, value, onChange, options }: {
  label: string; value: string; onChange: (v: string) => void; options: string[];
}) {
  const id = useId();
  return (
    <div className="fcc-field">
      <label className="fcc-label" htmlFor={id}>{label}</label>
      <select className="fcc-select" id={id} value={value} onChange={(e) => onChange(e.target.value)}>
        {options.map((option) => <option key={option} value={option}>{option}</option>)}
      </select>
    </div>
  );
}
