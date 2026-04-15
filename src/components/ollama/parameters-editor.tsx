import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ModelParameter } from "./modelfile-utils";
import "./ollama.css";

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
  const [params, setParams] = useState<ModelParameter[]>(
    initialParameters.length > 0 ? initialParameters : [{ key: "", value: "" }],
  );
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
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">
          {modelName} — {t("ollama.parameters")}
        </span>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="ollama-btn" onClick={onCancel} disabled={saving}>
            {t("ollama.cancel")}
          </button>
          <button
            className="ollama-btn ollama-btn-primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "..." : t("ollama.save")}
          </button>
        </div>
      </div>

      {error && (
        <div style={{
          padding: "var(--space-sm) var(--space-md)",
          color: "#e66", fontSize: "var(--text-xs)",
          background: "rgba(230,102,102,0.08)",
          borderBottom: "1px solid var(--edge)",
        }}>
          {error}
        </div>
      )}

      <div style={{
        flex: 1, overflow: "auto",
        padding: "var(--space-md)",
        display: "flex", flexDirection: "column", gap: 8,
      }}>
        {params.map((p, idx) => (
          <div key={idx} style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              value={p.key}
              onChange={(e) => updateRow(idx, "key", e.target.value)}
              placeholder="num_ctx"
              style={inputStyle(180)}
            />
            <input
              value={p.value}
              onChange={(e) => updateRow(idx, "value", e.target.value)}
              placeholder="32768"
              style={{ ...inputStyle(0), flex: 1 }}
            />
            <button
              className="ollama-btn"
              onClick={() => removeRow(idx)}
              style={{ padding: "6px 10px" }}
              title={t("ollama.remove")}
            >
              ×
            </button>
          </div>
        ))}

        <button
          className="ollama-btn"
          onClick={addRow}
          style={{ alignSelf: "flex-start", marginTop: 4 }}
        >
          {t("ollama.addParameter")}
        </button>

        <div style={{
          marginTop: "var(--space-md)",
          fontSize: "var(--text-xs)",
          color: "var(--ink-faint)",
          lineHeight: 1.6,
        }}>
          {t("ollama.commonParameters")} {t("ollama.commonParametersHint")}
        </div>
      </div>
    </div>
  );
}

function inputStyle(width: number): React.CSSProperties {
  return {
    width: width || undefined,
    padding: "6px 10px",
    background: "var(--void)",
    border: "1px solid var(--edge)",
    borderRadius: "var(--radius-sm)",
    color: "var(--ink)",
    fontSize: "var(--text-xs)",
    fontFamily: "var(--font-mono)",
    outline: "none",
  };
}
