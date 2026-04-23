import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke, Channel } from "@tauri-apps/api/core";
import "./ollama.css";

interface OllamaSetupProgress {
  completed: number;
  total: number;
  status: string;
}

interface OllamaSetupScreenProps {
  onComplete: () => void;
}

export function OllamaSetupScreen({ onComplete }: OllamaSetupScreenProps) {
  const { t } = useTranslation();
  const [downloading, setDownloading] = useState(false);
  const [percent, setPercent] = useState(0);
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleDownload = useCallback(async () => {
    setDownloading(true);
    setError(null);
    setPercent(0);

    const channel = new Channel<OllamaSetupProgress>();
    channel.onmessage = (event) => {
      setStatus(event.status);
      if (event.total > 0) {
        setPercent(Math.round((event.completed / event.total) * 100));
      }
    };

    try {
      await invoke("download_ollama", { onProgress: channel });
      onComplete();
    } catch (e) {
      setError(String(e));
      setDownloading(false);
    }
  }, [onComplete]);

  return (
    <div style={{
      display: "flex", flexDirection: "column", alignItems: "center",
      justifyContent: "center", height: "100%", gap: 24, padding: 40,
    }}>
      <h2 style={{
        fontSize: "var(--text-xl)", fontWeight: 700,
        color: "var(--ink)", margin: 0,
      }}>
        {t("ollamaSetup.title")}
      </h2>
      <p style={{
        fontSize: "var(--text-sm)", color: "var(--ink-muted)",
        textAlign: "center", maxWidth: 400, margin: 0,
      }}>
        {t("ollamaSetup.description")}
      </p>

      {downloading ? (
        <div style={{ width: 300, display: "flex", flexDirection: "column", gap: 8 }}>
          <div className="ollama-progress-bar" style={{ width: "100%" }}>
            <div className="ollama-progress-fill" style={{ width: `${percent}%` }} />
          </div>
          <span style={{
            fontSize: "var(--text-xs)", color: "var(--ink-faint)", textAlign: "center",
          }}>
            {status === "extracting" ? t("ollamaSetup.extracting") : `${percent}%`}
          </span>
        </div>
      ) : (
        <button
          className="ollama-btn ollama-btn-primary"
          style={{ padding: "8px 24px", fontSize: "var(--text-sm)" }}
          onClick={handleDownload}
        >
          {t("ollamaSetup.download")}
        </button>
      )}

      {error && (
        <span style={{ fontSize: "var(--text-xs)", color: "#e66", textAlign: "center" }}>
          {t("ollamaSetup.error")}
        </span>
      )}
    </div>
  );
}
