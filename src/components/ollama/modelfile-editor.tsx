import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import "./ollama.css";

interface ModelfileEditorProps {
  modelName: string;
  initialContent: string;
  onSave: (content: string) => void;
  onCancel: () => void;
}

export function ModelfileEditor({
  modelName, initialContent, onSave, onCancel,
}: ModelfileEditorProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(initialContent);
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("update_modelfile", { name: modelName, content });
      onSave(content);
    } catch (e: unknown) {
      console.error("Erreur sauvegarde modelfile:", e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">
          {modelName} — {t("ollama.editing")}
        </span>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="ollama-btn" onClick={onCancel}>{t("ollama.cancel")}</button>
          <button className="ollama-btn ollama-btn-primary" onClick={handleSave} disabled={saving}>
            {saving ? "..." : t("ollama.save")}
          </button>
        </div>
      </div>
      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        style={{
          flex: 1, padding: "var(--space-md)",
          fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)",
          background: "var(--void)", color: "var(--ink)",
          resize: "none", outline: "none", border: "none",
        }}
      />
    </div>
  );
}
