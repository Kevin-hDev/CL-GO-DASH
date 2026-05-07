import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ModelfileEditor } from "./modelfile-editor";
import { SystemPromptEditor } from "./system-prompt-editor";
import { ParametersEditor } from "./parameters-editor";
import { ModelfileView } from "./modelfile-view";
import { extractSystemPrompt, extractParameters } from "./modelfile-utils";

type Mode = "view" | "edit-system" | "edit-parameters" | "edit-modelfile";

interface ModelfileViewerProps {
  modelName: string;
  onDeleted?: () => void;
}

export function ModelfileViewer({ modelName, onDeleted }: ModelfileViewerProps) {
  const { t } = useTranslation();
  const [modelfile, setModelfile] = useState("");
  const [mode, setMode] = useState<Mode>("view");
  const [loading, setLoading] = useState(true);
  const [deleting, setDeleting] = useState(false);

  const handleDelete = async () => {
    setDeleting(true);
    try {
      await invoke("delete_ollama_model", { name: modelName });
      onDeleted?.();
    } catch (e: unknown) {
      console.warn("[ollama] delete model:", e);
    } finally {
      setDeleting(false);
    }
  };

  const systemPrompt = useMemo(() => extractSystemPrompt(modelfile), [modelfile]);
  const parameters = useMemo(() => extractParameters(modelfile), [modelfile]);

  const loadModelfile = () =>
    invoke<string>("get_modelfile", { name: modelName })
      .then(setModelfile)
      .catch(() => {});

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    setMode("view");
    void loadModelfile().finally(() => setLoading(false));
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
    <ModelfileView
      modelName={modelName}
      systemPrompt={systemPrompt}
      parameters={parameters}
      modelfile={modelfile}
      deleting={deleting}
      onDelete={() => void handleDelete()}
      onEditSystem={() => setMode("edit-system")}
      onEditParameters={() => setMode("edit-parameters")}
      onEditModelfile={() => setMode("edit-modelfile")}
    />
  );
}
