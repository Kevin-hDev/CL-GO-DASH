import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Tooltip } from "@/components/ui/tooltip";
import { ModelEditorShell } from "./model-editor-shell";
import type { ModelParameter } from "./modelfile-utils";
import "./ollama.css";
import "./parameters-editor.css";

interface ParametersEditorProps {
  modelName: string;
  initialParameters: ModelParameter[];
  onSave: () => void;
  onCancel: () => void;
}

export function ParametersEditor({
  modelName, initialParameters, onSave, onCancel,
}: ParametersEditorProps) {
  const { t } = useTranslation();
  const [params, setParams] = useState<ModelParameter[]>(() => {
    const base = initialParameters.length > 0 ? [...initialParameters] : [];
    const keys = new Set(base.map((p) => p.key));
    if (!keys.has("num_ctx")) base.push({ key: "num_ctx", value: "" });
    if (!keys.has("num_predict")) base.push({ key: "num_predict", value: "" });
    return base;
  });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const updateRow = (idx: number, field: "key" | "value", v: string) => {
    setParams((prev) => prev.map((p, i) => (i === idx ? { ...p, [field]: v } : p)));
  };

  const removeRow = (idx: number) => {
    setParams((prev) => prev.filter((_, i) => i !== idx));
  };

  const addRow = () => {
    setParams((prev) => [...prev, { key: "", value: "" }]);
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      const payload = params
        .filter((p) => p.key.trim() && p.value.trim())
        .map((p): [string, string] => [p.key.trim(), p.value.trim()]);
      await invoke("update_parameters", { name: modelName, parameters: payload });
      onSave();
    } catch {
      setError(t("errors.operationFailed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <ModelEditorShell
      title={`${modelName} — ${t("ollama.parameters")}`}
      cancelLabel={t("ollama.cancel")}
      saveLabel={t("ollama.save")}
      saving={saving}
      error={error}
      onCancel={onCancel}
      onSave={() => void handleSave()}
    >
      <div className="pe-body">
        {params.map((p, idx) => (
          <div key={idx} className="pe-row">
            <input
              value={p.key}
              onChange={(e) => updateRow(idx, "key", e.target.value)}
              placeholder="num_ctx"
              className="pe-input pe-input-key"
            />
            <input
              value={p.value}
              onChange={(e) => updateRow(idx, "value", e.target.value)}
              placeholder="32768"
              className="pe-input pe-input-value"
            />
            <Tooltip label={t("ollama.remove")}>
              <button
                className="ollama-btn pe-remove-btn"
                onClick={() => removeRow(idx)}
              >
                ×
              </button>
            </Tooltip>
          </div>
        ))}

        <button
          className="ollama-btn pe-add-btn"
          onClick={addRow}
        >
          {t("ollama.addParameter")}
        </button>

        <div className="pe-hint">
          {t("ollama.commonParameters")} {t("ollama.commonParametersHint")}
        </div>
      </div>
    </ModelEditorShell>
  );
}
