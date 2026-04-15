import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke, Channel } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import type { PullProgress } from "@/types/agent";
import "./ollama.css";

interface ModelInstallButtonProps {
  fullName: string;
  isInstalled: boolean;
  hasUpdate: boolean;
}

export function ModelInstallButton({
  fullName, isInstalled, hasUpdate,
}: ModelInstallButtonProps) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const [percent, setPercent] = useState(0);

  const handleAction = useCallback(async () => {
    setBusy(true);
    setStatus("Démarrage…");
    setPercent(0);

    const channel = new Channel<PullProgress>();
    channel.onmessage = (event: PullProgress) => {
      setStatus(event.status);
      if (event.total && event.completed) {
        setPercent(Math.round((event.completed / event.total) * 100));
      }
    };

    try {
      await invoke("pull_ollama_model", { name: fullName, onProgress: channel });
      setStatus("");
    } catch (e: unknown) {
      setStatus(`Erreur : ${e}`);
    } finally {
      setBusy(false);
    }
  }, [fullName]);

  if (busy) {
    return (
      <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 4 }}>
        <div className="ollama-progress-bar">
          <div className="ollama-progress-fill" style={{ width: `${percent}%` }} />
        </div>
        <span style={{ fontSize: "var(--text-xs)", color: "var(--ink-faint)" }}>
          {status}
        </span>
      </div>
    );
  }

  if (isInstalled && !hasUpdate) {
    return (
      <div
        title="Installé et à jour"
        style={{
          display: "flex", alignItems: "center",
          color: "var(--pulse)",
        }}
      >
        <Check size={18} />
      </div>
    );
  }

  if (isInstalled && hasUpdate) {
    return (
      <button
        className="ollama-btn ollama-btn-primary"
        onClick={handleAction}
        title="Une nouvelle version est disponible"
      >
        Update
      </button>
    );
  }

  return (
    <button className="ollama-btn ollama-btn-primary" onClick={handleAction}>
      {t("ollama.install")}
    </button>
  );
}
