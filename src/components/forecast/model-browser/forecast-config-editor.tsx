import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { SettingsSelect } from "@/components/settings/settings-select";
import type { ForecastModelEntry } from "../forecast-model-meta";
import type { ForecastConfigParam, ForecastModelConfig } from "./forecast-config-types";
import "@/components/ollama/ollama.css";
import "@/components/ollama/parameters-editor.css";
import "./forecast-config-editor.css";

interface ForecastConfigEditorProps {
  model: ForecastModelEntry | null;
}

export function ForecastConfigEditor({ model }: ForecastConfigEditorProps) {
  const { t } = useTranslation();
  const [config, setConfig] = useState<ForecastModelConfig | null>(null);
  const [draft, setDraft] = useState<Record<string, string>>({});
  const [editing, setEditing] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!model) return;
    void invoke<ForecastModelConfig>("get_forecast_model_config", { modelId: model.id })
      .then((next) => {
        setEditing(false);
        setConfig(next);
        setDraft(toDraft(next.params));
      })
      .catch(() => setConfig(null));
  }, [model]);

  const allParams = useMemo(
    () => [...(config?.params ?? []), ...(config?.inherited ?? [])],
    [config],
  );

  if (!model) {
    return <EmptyState message={t("forecast.modelConfig.noModel")} />;
  }

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const payload = buildPayload(config.params, draft);
      const next = await invoke<ForecastModelConfig>("set_forecast_model_config", {
        modelId: model.id,
        values: payload,
      });
      setConfig(next);
      setDraft(toDraft(next.params));
      setEditing(false);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fcfg-root">
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">
          {model.display_name} — {t("forecast.modelConfig.title")}
        </span>
        <div className="pe-actions">
          {editing && (
            <button className="ollama-btn" onClick={() => {
              setDraft(toDraft(config?.params ?? []));
              setEditing(false);
            }} disabled={saving}>
              {t("forecast.modelConfig.cancel")}
            </button>
          )}
          <button
            className={`ollama-btn ${editing ? "ollama-btn-primary" : ""}`}
            onClick={() => editing ? void handleSave() : setEditing(true)}
            disabled={saving || !config}
          >
            {saving ? "..." : t(editing ? "forecast.modelConfig.save" : "forecast.modelConfig.edit")}
          </button>
        </div>
      </div>

      <div className="fcfg-body">
        <div className="fcfg-rows">
          {config?.params.map((param) => (
            <ConfigRow
              key={param.id}
              param={param}
              value={draft[param.id] ?? ""}
              editing={editing}
              onChange={(value) => setDraft((prev) => ({ ...prev, [param.id]: value }))}
            />
          ))}
          {config?.inherited.map((param) => (
            <ConfigRow
              key={param.id}
              param={param}
              value={valueToText(param.effective_value)}
              editing={false}
              inherited
              onChange={() => undefined}
            />
          ))}
        </div>

        <div className="fcfg-descriptions">
          {allParams.map((param) => (
            <div key={param.id} className="fcfg-description">
              <span>{t(param.label_key)}</span>
              <p>{t(param.description_key)}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function ConfigRow({
  param, value, editing, inherited, onChange,
}: {
  param: ForecastConfigParam;
  value: string;
  editing: boolean;
  inherited?: boolean;
  onChange: (value: string) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="fcfg-row">
      <div className="fcfg-key">{t(param.label_key)}</div>
      <div className="fcfg-value">
        {param.kind === "boolean" || param.kind === "select" ? (
          <SettingsSelect
            value={value || valueToText(param.default_value)}
            onChange={onChange}
            options={selectOptions(param, t)}
            disabled={!editing}
          />
        ) : (
          <input
            className="pe-input pe-input-value"
            value={value}
            disabled={!editing}
            placeholder={valueToText(param.default_value)}
            onChange={(event) => onChange(event.target.value)}
          />
        )}
      </div>
      <div className="fcfg-default">
        {inherited ? t("forecast.modelConfig.inherited") : valueToText(param.default_value)}
      </div>
    </div>
  );
}

function selectOptions(param: ForecastConfigParam, t: (key: string) => string) {
  if (param.kind === "boolean") {
    return [
      { value: "true", label: t("forecast.modelConfig.enabled") },
      { value: "false", label: t("forecast.modelConfig.disabled") },
    ];
  }
  return param.options.map((option) => ({ value: option, label: option }));
}

function toDraft(params: ForecastConfigParam[]): Record<string, string> {
  return Object.fromEntries(params.map((param) => [param.id, valueToText(param.value)]));
}

function buildPayload(params: ForecastConfigParam[], draft: Record<string, string>) {
  return Object.fromEntries(params.map((param) => [param.id, toPayloadValue(param, draft[param.id])]));
}

function toPayloadValue(param: ForecastConfigParam, value: string | undefined) {
  if (!value) return null;
  if (param.kind === "boolean") return value === "true";
  return value;
}

function valueToText(value: unknown): string {
  if (Array.isArray(value)) return value.join(", ");
  if (typeof value === "boolean") return value ? "true" : "false";
  if (typeof value === "number" || typeof value === "string") return String(value);
  if (value === null || value === undefined) return "";
  return "";
}
