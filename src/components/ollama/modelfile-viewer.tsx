import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ModelfileEditor } from "./modelfile-editor";
import { SystemPromptEditor } from "./system-prompt-editor";
import { ParametersEditor } from "./parameters-editor";
import { extractSystemPrompt, extractParameters } from "./modelfile-utils";
import "./ollama.css";

type Mode = "view" | "edit-system" | "edit-parameters" | "edit-modelfile";

interface ModelfileViewerProps {
  modelName: string;
}

export function ModelfileViewer({ modelName }: ModelfileViewerProps) {
  const { t } = useTranslation();
  const [modelfile, setModelfile] = useState("");
  const [mode, setMode] = useState<Mode>("view");
  const [loading, setLoading] = useState(true);

  const systemPrompt = useMemo(() => extractSystemPrompt(modelfile), [modelfile]);
  const parameters = useMemo(() => extractParameters(modelfile), [modelfile]);

  const loadModelfile = () => {
    invoke<string>("get_modelfile", { name: modelName })
      .then(setModelfile)
      .catch((e: unknown) => console.warn("Erreur chargement modelfile:", e));
  };

  useEffect(() => {
    setLoading(true);
    setMode("view");
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

  if (mode === "edit-system") {
    return (
      <SystemPromptEditor
        modelName={modelName}
        initialSystem={systemPrompt}
        onSave={() => { setMode("view"); loadModelfile(); }}
        onCancel={() => setMode("view")}
      />
    );
  }

  if (mode === "edit-parameters") {
    return (
      <ParametersEditor
        modelName={modelName}
        initialParameters={parameters}
        onSave={() => { setMode("view"); loadModelfile(); }}
        onCancel={() => setMode("view")}
      />
    );
  }

  if (mode === "edit-modelfile") {
    return (
      <ModelfileEditor
        modelName={modelName}
        initialContent={modelfile}
        onSave={(c) => { setModelfile(c); setMode("view"); }}
        onCancel={() => setMode("view")}
      />
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="ollama-detail-header">
        <span className="ollama-detail-name">{modelName}</span>
        <button className="ollama-btn" onClick={() => setMode("edit-modelfile")}>
          Éditer modelfile
        </button>
      </div>

      <Section title="System prompt" onEdit={() => setMode("edit-system")}>
        <div style={{
          fontSize: "var(--text-sm)",
          color: systemPrompt ? "var(--ink)" : "var(--ink-faint)",
          whiteSpace: "pre-wrap", lineHeight: 1.5,
          fontStyle: systemPrompt ? "normal" : "italic",
          maxHeight: 200, overflow: "auto",
        }}>
          {systemPrompt || "(aucun system prompt défini)"}
        </div>
      </Section>

      <Section title="Paramètres" onEdit={() => setMode("edit-parameters")}>
        {parameters.length === 0 ? (
          <div style={{ fontStyle: "italic", color: "var(--ink-faint)", fontSize: "var(--text-sm)" }}>
            (aucun paramètre — Ollama utilise ses valeurs par défaut)
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {parameters.map((p, i) => (
              <div key={i} style={{
                display: "flex", gap: 12,
                fontSize: "var(--text-xs)",
                fontFamily: "var(--font-mono)",
              }}>
                <span style={{ color: "var(--ink-muted)", minWidth: 140 }}>{p.key}</span>
                <span style={{ color: "var(--ink)" }}>{p.value}</span>
              </div>
            ))}
          </div>
        )}
      </Section>

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

function Section({
  title, onEdit, children,
}: { title: string; onEdit: () => void; children: React.ReactNode }) {
  return (
    <div style={{ padding: "var(--space-md)", borderBottom: "1px solid var(--edge)" }}>
      <div style={{
        display: "flex", alignItems: "center",
        justifyContent: "space-between", marginBottom: "var(--space-sm)",
      }}>
        <span style={{
          fontSize: "var(--text-xs)", color: "var(--ink-faint)",
          textTransform: "uppercase", letterSpacing: "0.05em",
        }}>
          {title}
        </span>
        <button className="ollama-btn ollama-btn-primary" onClick={onEdit}>
          Éditer
        </button>
      </div>
      {children}
    </div>
  );
}
