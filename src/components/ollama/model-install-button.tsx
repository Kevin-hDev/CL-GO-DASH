import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { PullProgress } from "@/types/agent";
import "./ollama.css";

interface ModelInstallButtonProps {
  modelName: string;
}

export function ModelInstallButton({ modelName }: ModelInstallButtonProps) {
  const { t } = useTranslation();
  const [installing, setInstalling] = useState(false);
  const [status, setStatus] = useState("");
  const [percent, setPercent] = useState(0);

  const handleInstall = useCallback(async () => {
    setInstalling(true);
    setStatus("Démarrage...");

    const channel = new Channel<PullProgress>();
    channel.onmessage = (event: PullProgress) => {
      setStatus(event.status);
      if (event.total && event.completed) {
        setPercent(Math.round((event.completed / event.total) * 100));
      }
    };

    try {
      await invoke("pull_ollama_model", { name: modelName, onProgress: channel });
      setStatus(t("ollama.install"));
    } catch (e: unknown) {
      setStatus(`Erreur: ${e}`);
    } finally {
      setInstalling(false);
    }
  }, [modelName, t]);

  if (installing) {
    return (
      <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 4 }}>
        <div className="ollama-progress-bar">
          <div className="ollama-progress-fill" style={{ width: `${percent}%` }} />
        </div>
        <span style={{ fontSize: "var(--text-xs)", color: "var(--ink-faint)" }}>{status}</span>
      </div>
    );
  }

  return (
    <button className="ollama-btn ollama-btn-primary" onClick={handleInstall}>
      {t("ollama.install")}
    </button>
  );
}
