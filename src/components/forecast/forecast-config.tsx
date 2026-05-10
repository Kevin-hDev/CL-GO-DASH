import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./forecast-config.css";

interface ModelEntry {
  id: string;
  display_name: string;
  provider_id: string;
  is_cloud: boolean;
  installed: boolean;
}

interface ForecastConfigProps {
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
  { value: "D", label: "Jour" },
  { value: "W", label: "Semaine" },
  { value: "M", label: "Mois" },
  { value: "Q", label: "Trimestre" },
  { value: "Y", label: "Année" },
  { value: "H", label: "Heure" },
  { value: "T", label: "Minute" },
];

export function ForecastConfig({ onLaunch, onBack }: ForecastConfigProps) {
  const [models, setModels] = useState<ModelEntry[]>([]);
  const [target, setTarget] = useState("");
  const [dateCol, setDateCol] = useState("");
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
        <button className="fcc-back" onClick={onBack}>← Retour</button>
        <span className="fcc-title">Configuration</span>
      </div>
      <div className="fcc-form">
        <Field label="Colonne cible" value={target} onChange={setTarget} placeholder="ex: revenue" />
        <Field label="Colonne date" value={dateCol} onChange={setDateCol} placeholder="ex: date" />
        <div className="fcc-row">
          <div className="fcc-field fcc-half">
            <label className="fcc-label" htmlFor="fcc-horizon">Horizon</label>
            <input className="fcc-input" id="fcc-horizon" type="number" min={1} max={5000}
              value={horizon} onChange={(e) => setHorizon(Number(e.target.value))} />
          </div>
          <div className="fcc-field fcc-half">
            <label className="fcc-label" htmlFor="fcc-freq">Fréquence</label>
            <select className="fcc-select" id="fcc-freq" value={frequency} onChange={(e) => setFrequency(e.target.value)}>
              {FREQUENCIES.map((f) => <option key={f.value} value={f.value}>{f.label}</option>)}
            </select>
          </div>
        </div>
        <div className="fcc-field">
          <label className="fcc-label" htmlFor="fcc-model">Modèle</label>
          <select className="fcc-select" id="fcc-model" value={model} onChange={(e) => setModel(e.target.value)}>
            <optgroup label="Local">
              {localModels.map((m) => (
                <option key={m.id} value={m.id} disabled={!m.installed}>
                  {m.display_name} {m.installed ? "" : "(non installé)"}
                </option>
              ))}
            </optgroup>
            <optgroup label="Cloud">
              {cloudModels.map((m) => (
                <option key={m.id} value={m.id}>{m.display_name}</option>
              ))}
            </optgroup>
          </select>
        </div>
        <div className="fcc-field">
          <label className="fcc-label" htmlFor="fcc-confidence">Confiance: {Math.round(confidence * 100)}%</label>
          <input className="fcc-range" id="fcc-confidence" type="range" min={0.5} max={0.99} step={0.01}
            value={confidence} onChange={(e) => setConfidence(Number(e.target.value))} />
        </div>
      </div>
      <div className="fcc-footer">
        <button className="fcc-launch" disabled={!canLaunch}
          onClick={() => onLaunch({ targetColumn: target, dateColumn: dateCol, horizon, frequency, model, confidence })}>
          Lancer le forecast
        </button>
      </div>
    </div>
  );
}

let fieldCounter = 0;
function Field({ label, value, onChange, placeholder }: {
  label: string; value: string; onChange: (v: string) => void; placeholder?: string;
}) {
  const [id] = useState(() => `fcc-field-${++fieldCounter}`);
  return (
    <div className="fcc-field">
      <label className="fcc-label" htmlFor={id}>{label}</label>
      <input className="fcc-input" id={id} value={value} onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder} />
    </div>
  );
}
