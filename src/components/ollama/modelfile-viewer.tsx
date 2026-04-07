import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ModelfileEditor } from "./modelfile-editor";
import "./ollama.css";

interface ModelfileViewerProps {
  modelName: string;
}

export function ModelfileViewer({ modelName }: ModelfileViewerProps) {
  const { t } = useTranslation();
  const [modelfile, setModelfile] = useState("");
  const [editing, setEditing] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    invoke<string>("get_modelfile", { name: modelName })
      .then(setModelfile)
      .catch((e: unknown) => console.warn("Erreur chargement modelfile:", e))
      .finally(() => setLoading(false));
  }, [modelName]);

  if (loading) {
    return (
      <div style={{ padding: "var(--space-md)", fontSize: "var(--text-sm)", color: "var(--ink-faint)" }}>
        {t("history.loading")}
      </div>
    );
  }

  if (editing) {
    return (
      <ModelfileEditor
        modelName={modelName}
        initialContent={modelfile}
        onSave={(c) => { setModelfile(c); setEditing(false); }}
        onCancel={() => setEditing(false)}
      />
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">{modelName}</span>
        <button className="ollama-btn" onClick={() => setEditing(true)}>
          {t("ollama.edit")}
        </button>
      </div>
      <pre style={{
        flex: 1, overflow: "auto", padding: "var(--space-md)",
        fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)",
        color: "var(--ink-muted)", whiteSpace: "pre-wrap", margin: 0,
      }}>
        {modelfile}
      </pre>
    </div>
  );
}
