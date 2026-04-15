import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "./ollama.css";

interface SystemPromptEditorProps {
  modelName: string;
  initialSystem: string;
  onSave: (system: string) => void;
  onCancel: () => void;
}

export function SystemPromptEditor({
  modelName, initialSystem, onSave, onCancel,
}: SystemPromptEditorProps) {
  const { t } = useTranslation();
  const [system, setSystem] = useState(initialSystem);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await invoke("update_system_prompt", { name: modelName, system });
      onSave(system);
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
          {modelName} — System prompt
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
      <textarea
        value={system}
        onChange={(e) => setSystem(e.target.value)}
        placeholder="Définis le comportement et la personnalité du modèle…"
        style={{
          flex: 1, padding: "var(--space-md)",
          fontSize: "var(--text-sm)", fontFamily: "var(--font-sans)",
          background: "var(--void)", color: "var(--ink)",
          resize: "none", outline: "none", border: "none",
          lineHeight: 1.6,
        }}
      />
    </div>
  );
}
